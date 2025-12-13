import java.io.ByteArrayOutputStream
import org.gradle.kotlin.dsl.support.serviceOf
import org.gradle.process.ExecOperations

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
}

// UPX compression toggle (override with -PuseUpx=false)
val useUpx: Boolean = (findProperty("useUpx") as String?)?.toBoolean() ?: false
val upxExecutable = providers.environmentVariable("UPX").orElse("upx")
val execOps: ExecOperations = serviceOf()

// ABI selection via -Pabi (arm64, armv7, both). Defaults to arm64 only.
val abiProp = (findProperty("abi") as String?)?.lowercase()
val selectedAbis = when (abiProp) {
    "armv7" -> listOf("armeabi-v7a")
    "both", "all" -> listOf("arm64-v8a", "armeabi-v7a")
    else -> listOf("arm64-v8a")
}

android {
    namespace = "aeska.kistaverk"
    compileSdk {
        version = release(36)
    }

    defaultConfig {
        applicationId = "aeska.kistaverk"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = "0.0.0-dev"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"


    }

    buildFeatures {
        buildConfig = false
    }

    packaging {
        resources {
            excludes += listOf(
                "META-INF/DEPENDENCIES",
                "META-INF/LICENSE",
                "META-INF/LICENSE.txt",
                "META-INF/license.txt",
                "META-INF/NOTICE",
                "META-INF/NOTICE.txt",
                "META-INF/notice.txt",
                "META-INF/AL2.0",
                "META-INF/LGPL2.1"
            )
        }
        jniLibs {
            // We strip before UPX; avoid a second strip pass that breaks packed libs.
            keepDebugSymbols += "**/libkistaverk_core.so"
        }
    }

    bundle {
        // Use Play-split App Bundle to trim download size
        abi {
            enableSplit = true
        }
        // Keep language resources together to avoid split install churn
        language {
            enableSplit = false
        }
    }

    splits {
        abi {
            isEnable = true
            reset()
            include(*selectedAbis.toTypedArray())
            isUniversalApk = false
        }
    }

    // Tell Gradle where to find the compiled .so files
    sourceSets.getByName("main") {
        jniLibs.srcDir("src/main/jniLibs")
    }

    tasks.register("cargoBuild") {
        val currentProject = project
        // 1. Find Rust (Keeping your search logic, which is good)
         val possibleLocations = listOf(
            file("../rust"),        // If rust is a sibling of 'app' (standard case)
            file("../../rust"),     // If rust is an uncle of 'app' (case 'app/app')
            file("rust")            // If rust is inside 'app'
        )
        val foundRustDir = possibleLocations.find { it.exists() && it.isDirectory }

        if (foundRustDir == null) {
            throw GradleException("❌ DOSSIER RUST INTROUVABLE.")
        }
        val rustDir = foundRustDir.canonicalFile
        val jniLibsDir = File(projectDir, "src/main/jniLibs")
        val enablePrecisionProvider = providers.provider {
            (currentProject.findProperty("enablePrecision") as String?)?.toBoolean() ?: false
        }

        // Resolve cargo from PATH (portable)
        val cargoPath = System.getenv("CARGO") ?: "cargo"

        doLast {
            // Create the directory if it doesn't exist
            if (!jniLibsDir.exists()) {
                jniLibsDir.mkdirs()
            }
            
            // Remove stale ABI outputs not being built this run
            jniLibsDir.listFiles()?.forEach { abiDir ->
                if (abiDir.isDirectory && !selectedAbis.contains(abiDir.name)) {
                    abiDir.deleteRecursively()
                }
            }

            // Define the architectures to build for, mapping Android ABI to Rust target and lib folder name
            val architectures = listOfNotNull(
                if (selectedAbis.contains("arm64-v8a")) Triple("arm64-v8a", "aarch64-linux-android", "aarch64-linux-android") else null,
                if (selectedAbis.contains("armeabi-v7a")) Triple("armeabi-v7a", "armv7-linux-androideabi", "armv7a-linux-androideabi") else null
            )

            // Check if we should enable precision feature
            val enablePrecision = enablePrecisionProvider.get()
            val precisionFeatureArg = if (enablePrecision) "precision" else ""

            architectures.forEach { (androidAbi, rustTarget, libArchFolder) ->
                println("🔨 Building Rust library for Android ABI: $androidAbi (Rust target: $rustTarget)...")
                
                val gmpLibsDir = File(rustDir, "libs/android/$libArchFolder/lib")
                val gmpIncludeDir = File(rustDir, "libs/android/$libArchFolder/include")

                // Ensure the architecture-specific jniLibs directory exists
                val currentAbiJniLibsDir = File(jniLibsDir, androidAbi)
                if (!currentAbiJniLibsDir.exists()) {
                    currentAbiJniLibsDir.mkdirs()
                }

                // Base command line arguments for cargo ndk build
                val baseArgs = mutableListOf(
                    "ndk",
                    "-t", androidAbi, // Use Android ABI here
                    "-o", currentAbiJniLibsDir.absolutePath, // Output to ABI-specific folder
                    "build", 
                    "--release",
                    "--target", rustTarget // Explicitly pass the Rust target
                )

                // Add precision feature if enabled
                if (precisionFeatureArg.isNotBlank()) {
                    baseArgs.add("--features")
                    baseArgs.add(precisionFeatureArg)
                }

                // Run: cargo ndk ...
                execOps.exec {
                    workingDir = rustDir
                    executable = cargoPath
                    val ndkDir = android.ndkDirectory
                    environment("ANDROID_NDK_HOME", ndkDir.absolutePath)
                    environment("PATH", System.getenv("PATH") + ":${System.getProperty("user.home")}/.cargo/bin")
                    environment(
                        "RUSTFLAGS",
                        "-C link-arg=-Wl,--gc-sections -C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-init=_init"
                    )
                    environment("CFLAGS", "-Os")
                    environment("GMP_LIB_DIR", gmpLibsDir.absolutePath)
                    environment("GMP_INCLUDE_DIR", gmpIncludeDir.absolutePath)
                    environment("GMP_STATIC", "1")
                    environment("MPFR_LIB_DIR", gmpLibsDir.absolutePath)
                    environment("MPFR_INCLUDE_DIR", gmpIncludeDir.absolutePath)
                    environment("MPFR_STATIC", "1")
                    environment("MPC_LIB_DIR", gmpLibsDir.absolutePath)
                    environment("MPC_INCLUDE_DIR", gmpIncludeDir.absolutePath)
                    environment("MPC_STATIC", "1")
                    environment("GMP_MPFR_SYS_USE_PKG_CONFIG", "0")
                    commandLine = listOf(cargoPath) + baseArgs
                }

                // Overwrite with unstripped artifact from target dir to preserve init array
                val builtLib = File(rustDir, "target/$rustTarget/release/libkistaverk_core.so")
                if (builtLib.exists()) {
                    builtLib.copyTo(File(currentAbiJniLibsDir, builtLib.name), overwrite = true)
                }
                // Clean cargo-ndk nested output if present (e.g., jniLibs/abi/abi/lib*.so)
                val nested = File(currentAbiJniLibsDir, androidAbi)
                if (nested.exists()) {
                    nested.deleteRecursively()
                }
                println("✅ Built for $androidAbi (Rust target: $rustTarget). Output in ${currentAbiJniLibsDir.absolutePath}")
            }
            // Clean up the temporary config file if it was created.
            // The temporary config file logic has been replaced by direct environment variables.
            // So no cleanup needed here.
        }
    }

// Hook the build task: Run cargoBuild before Android processes resources
    tasks.withType<com.android.build.gradle.tasks.MergeSourceSetFolders>().configureEach {
        dependsOn("cargoBuild")
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
    ndkVersion = "29.0.14206865"
    buildToolsVersion = "36.0.0"
}



dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity)
    implementation(libs.androidx.documentfile)

    // CameraX
    implementation(libs.androidx.camera.core)
    implementation(libs.androidx.camera.camera2)
    implementation(libs.androidx.camera.lifecycle)
    implementation(libs.androidx.camera.view)

    testImplementation(libs.junit)
    testImplementation(libs.robolectric)
    testImplementation(libs.androidx.test.core)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}

// Generate deps metadata from Cargo before build
tasks.register<Exec>("generateDepsMetadata") {
    group = "build"
    description = "Generate deps.json from cargo metadata"
    val repoRoot = rootProject.rootDir.parentFile
    workingDir = File(repoRoot, "rust")
    commandLine = listOf("./scripts/generate_deps_metadata.sh")
    doFirst {
        println("Generating deps metadata in ${workingDir.absolutePath}")
    }
}

// Task to build with precision feature enabled
tasks.register("buildWithPrecision") {
    group = "build"
    description = "Build Rust library with precision feature (arbitrary precision arithmetic)"
    
    dependsOn("cargoBuild")
    
    doFirst {
        project.ext.set("enablePrecision", true)
        println("🚀 Building with precision feature enabled")
        println("   This will enable arbitrary precision arithmetic using GMP/MPFR/MPC")
        println("   Note: Requires prebuilt Android libraries from scripts/build_gmp_android.sh")
    }
}

// Task to build without precision feature (default)
tasks.register("buildWithoutPrecision") {
    group = "build"
    description = "Build Rust library without precision feature (standard f64 arithmetic)"
    
    dependsOn("cargoBuild")
    
    doFirst {
        project.ext.set("enablePrecision", false)
        println("⚡ Building without precision feature (default)")
        println("   This uses standard f64 arithmetic - faster build, smaller APK")
    }
}

// Compress built JNI libs with UPX (opt-in via -PuseUpx=true)
tasks.register("compressJniWithUpx") {
    group = "build"
    description = "Compress JNI .so files with UPX (requires UPX installed)"
    dependsOn("cargoBuild")
    onlyIf { useUpx }

    doLast {
        val upxName = upxExecutable.get()
        val upxPath = run {
            val candidate = File(upxName)
            if (candidate.isAbsolute && candidate.canExecute()) candidate
            else {
                val pathEnv = System.getenv("PATH") ?: ""
                pathEnv.split(File.pathSeparator)
                    .map { File(it, upxName) }
                    .firstOrNull { it.canExecute() }
            }
        } ?: throw GradleException("UPX not found on PATH. Install UPX or rerun with -PuseUpx=false / USE_UPX=false.")

        val jniLibsRoot = File(projectDir, "src/main/jniLibs")
        if (!jniLibsRoot.exists()) {
            throw GradleException("JNI libs directory not found: ${jniLibsRoot.absolutePath}. Ensure cargoBuild ran.")
        }

        val libs = fileTree(jniLibsRoot) {
            include(*selectedAbis.map { "$it/**/*.so" }.toTypedArray())
        }.files
        if (libs.isEmpty()) {
            logger.lifecycle("compressJniWithUpx: no .so files found under ${jniLibsRoot.absolutePath}")
            return@doLast
        }

        libs.forEach { lib ->
            logger.lifecycle("UPX compressing ${lib.absolutePath}")
            val before = lib.length()
            val output = ByteArrayOutputStream()
            val result = execOps.exec {
                // Ensure executable bit for UPX
                lib.setExecutable(true, false)
                // Strip debug symbols before packing to avoid Gradle strip failures later
                val ndkDir = android.ndkDirectory
                val stripTool = File(ndkDir, "toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip")
                if (stripTool.canExecute()) {
                    execOps.exec {
                        commandLine(stripTool.absolutePath, "--strip-unneeded", lib.absolutePath)
                    }
                } else {
                    logger.warn("llvm-strip not found/executable at ${stripTool.absolutePath}, skipping pre-strip")
                }
                commandLine(upxPath.absolutePath, "--best", "--lzma", lib.absolutePath)
                isIgnoreExitValue = true
                standardOutput = output
                errorOutput = output
            }
            if (result.exitValue != 0) {
                throw GradleException("UPX failed for ${lib.name}: ${output.toString().trim()}")
            }
            val after = lib.length()
            if (before > 0 && after > 0) {
                val saved = before - after
                val pct = (saved.toDouble() * 100.0 / before.toDouble()).let { String.format("%.2f", it) }
                logger.lifecycle("UPX saved ${saved} bytes (${pct}%) on ${lib.name} (from $before to $after)")
            }
        }
    }
}

tasks.named("preBuild") {
    dependsOn("generateDepsMetadata")
    dependsOn("compressJniWithUpx")
}
