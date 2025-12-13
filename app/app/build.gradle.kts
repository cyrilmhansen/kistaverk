plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
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
        versionName = "1.0"

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
            include("arm64-v8a", "armeabi-v7a")
            isUniversalApk = false
        }
    }

    // Tell Gradle where to find the compiled .so files
    sourceSets.getByName("main") {
        jniLibs.srcDir("src/main/jniLibs")
    }

    tasks.register("cargoBuild") {
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

        // 2. Define the destination as ABSOLUTE (No more ../..)
        // "this.projectDir" always points to the module directory (the 2nd 'app')
        val jniLibsDir = File(projectDir, "src/main/jniLibs")

        // Resolve cargo from PATH (portable)
        val cargoPath = System.getenv("CARGO") ?: "cargo"
        val ndkDir = android.ndkDirectory

        doLast {
            // Create the directory if it doesn't exist
            if (!jniLibsDir.exists()) {
                jniLibsDir.mkdirs()
            }
            
            // Define the architectures to build for, mapping Android ABI to Rust target and lib folder name
            val architectures = listOf(
                Triple("arm64-v8a", "aarch64-linux-android", "aarch64-linux-android"),
                Triple("armeabi-v7a", "armv7-linux-androideabi", "armv7a-linux-androideabi")
            )

            // Check if we should enable precision feature
            val enablePrecision = project.ext.has("enablePrecision") && 
                                  project.ext.get("enablePrecision").toString().toBoolean()
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

                exec {
                    workingDir = rustDir
                    executable = cargoPath
                    // Common environment variables
                    environment("ANDROID_NDK_HOME", ndkDir.absolutePath)
                    environment("PATH", System.getenv("PATH") + ":${System.getProperty("user.home")}/.cargo/bin")
                    environment("RUSTFLAGS", "-C link-arg=-Wl,--gc-sections -C link-arg=-Wl,-z,max-page-size=16384")
                    environment("CFLAGS", "-Os")
                    
                    // GMP/MPFR/MPC environment variables, dynamically set per architecture
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
                    
                    commandLine = baseArgs
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

tasks.named("preBuild") {
    dependsOn("generateDepsMetadata")
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
