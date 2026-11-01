plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

val staged = layout.buildDirectory.dir("native-input")
val prototypeAbi = providers.gradleProperty("prototypeAbi").orElse("arm64-v8a")
require(prototypeAbi.get() in listOf("arm64-v8a", "x86_64"))
val prototypeCode = providers.gradleProperty("prototypeVersionCode").orElse("1")

android {
    namespace = "io.github.alexandrezanata.tbg.prototype"
    compileSdk = 36
    buildToolsVersion = "36.0.0"
    ndkVersion = "28.2.13676358"

    defaultConfig {
        applicationId = "io.github.alexandrezanata.tbg.prototype"
        minSdk = 31
        targetSdk = 36
        versionCode = prototypeCode.get().toInt()
        versionName = "0.1.1-rc.1-android.2"
        ndk { abiFilters += prototypeAbi.get() }
    }

    // A private, stable test key is supplied locally; never silently change signatures.
    signingConfigs {
        create("prototype") {
            storeFile = providers.environmentVariable("TBG_ANDROID_KEYSTORE").orNull?.let(::file)
            storePassword = providers.environmentVariable("TBG_ANDROID_STORE_PASSWORD").orNull
            keyAlias = providers.environmentVariable("TBG_ANDROID_KEY_ALIAS").orNull
            keyPassword = providers.environmentVariable("TBG_ANDROID_KEY_PASSWORD").orNull
        }
    }
    buildTypes {
        debug { signingConfig = signingConfigs.getByName("prototype") }
        release {
            signingConfig = signingConfigs.getByName("prototype")
            isMinifyEnabled = false
        }
    }

    sourceSets["main"].apply {
        assets.srcDir(staged.map { it.dir("assets") })
        jniLibs.srcDir(staged.map { it.dir("jniLibs") })
        res.srcDir("../../../art/branding/android/res")
    }
    packaging {
        // Page-aligned, uncompressed libraries can be mapped directly from the APK.
        jniLibs.useLegacyPackaging = false
        // Strip explicitly before provenance/staging; Gradle must preserve the verified bytes.
        jniLibs.keepDebugSymbols += "**/*.so"
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {
    // Matches android-activity 0.6.1's bundled C++ glue; do not enable Prefab.
    implementation("androidx.games:games-activity:4.4.0")
    // GameActivity's published POM omits its AppCompat superclass/resources.
    implementation("androidx.appcompat:appcompat:1.7.1")
}

val verifyNativeInput by tasks.registering(Exec::class) {
    workingDir(rootDir.resolve("../.."))
    commandLine("python3", "scripts/package-android.py", "verify-stage", "--abi", prototypeAbi.get())
}

// Host-only Kotlin compilation can proceed independently. Every APK/AAB must
// merge the verified native input; Gradle never launches Cargo implicitly.
tasks.configureEach {
    if (name.startsWith("merge") && name.endsWith("NativeLibs")) {
        dependsOn(verifyNativeInput)
    }
}
