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


        ndk {
            // Tell Gradle which ABIs we support
            abiFilters.add("arm64-v8a")
            abiFilters.add("armeabi-v7a")
            abiFilters.add("x86_64")
        }
    }

    // Tell Gradle where to find the compiled .so files
    sourceSets.getByName("main") {
        jniLibs.srcDir("src/main/jniLibs")
    }

    tasks.register<Exec>("cargoBuild") {
        // 1. Trouver Rust (On garde ta logique de recherche qui est bonne)
        val possibleLocations = listOf(
            file("../rust"),        // Si rust est frère de 'app' (cas standard)
            file("../../rust"),     // Si rust est oncle de 'app' (cas 'app/app')
            file("rust")            // Si rust est dans 'app'
        )
        val foundRustDir = possibleLocations.find { it.exists() && it.isDirectory }

        if (foundRustDir == null) {
            throw GradleException("❌ DOSSIER RUST INTROUVABLE.")
        }
        val rustDir = foundRustDir.canonicalFile

        // 2. Définir la destination en ABSOLU (Fini les ../..)
        // "this.projectDir" pointe toujours vers le dossier du module (le 2ème 'app')
        val jniLibsDir = File(projectDir, "src/main/jniLibs")

        // Configuration
        workingDir = rustDir
        executable = "/usr/bin/cargo"
        val ndkDir = android.ndkDirectory
        environment("ANDROID_NDK_HOME", ndkDir.absolutePath)
        environment("PATH", System.getenv("PATH") + ":${System.getProperty("user.home")}/.cargo/bin")

        args(
            "ndk",
            "-t", "armeabi-v7a",
            "-t", "arm64-v8a",
            "-t", "x86_64",
            "-o", jniLibsDir.absolutePath, // <--- ICI : Chemin absolu garanti !
            "build", "--release"
        )

        doFirst {
            println("✅ Rust source : ${rustDir.absolutePath}")
            println("✅ Destination libs : ${jniLibsDir.absolutePath}")

            // Création du dossier s'il n'existe pas (pour éviter que cargo râle)
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
            isMinifyEnabled = false
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
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
    implementation(libs.androidx.activity)
    implementation(libs.androidx.constraintlayout)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}