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
            include("arm64-v8a")
            isUniversalApk = false
        }
    }

    // Tell Gradle where to find the compiled .so files
    sourceSets.getByName("main") {
        jniLibs.srcDir("src/main/jniLibs")
    }

    tasks.register<Exec>("cargoBuild") {
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

        // Configuration
        workingDir = rustDir
        executable = cargoPath
        val ndkDir = android.ndkDirectory
        environment("ANDROID_NDK_HOME", ndkDir.absolutePath)
        environment("PATH", System.getenv("PATH") + ":${System.getProperty("user.home")}/.cargo/bin")
        environment("RUSTFLAGS", "-C link-arg=-Wl,--gc-sections")

        args(
            "ndk",
            "-t", "arm64-v8a",
            "-o", jniLibsDir.absolutePath, // <--- HERE: Absolute path guaranteed!
            "build", "--release"
        )

        doFirst {
            println("✅ Rust source : ${rustDir.absolutePath}")
            println("✅ Destination libs : ${jniLibsDir.absolutePath}")

        // Create the directory if it doesn't exist (to prevent cargo from complaining)
            if (!jniLibsDir.exists()) {
                jniLibsDir.mkdirs()
            }
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
    testImplementation(libs.junit)
    testImplementation(libs.robolectric)
    testImplementation(libs.androidx.test.core)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}
