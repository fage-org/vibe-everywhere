import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

val keystoreProperties = Properties().apply {
    val propFile = file("keystore.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

fun signingValue(envName: String, propertyName: String): String? {
    val envValue = providers.environmentVariable(envName).orNull?.trim()
    if (!envValue.isNullOrEmpty()) {
        return envValue
    }

    return keystoreProperties.getProperty(propertyName)?.trim()?.takeIf { it.isNotEmpty() }
}

val releaseSigningStoreFile = signingValue("VIBE_ANDROID_KEYSTORE_PATH", "storeFile")
val releaseSigningStorePassword = signingValue("VIBE_ANDROID_KEYSTORE_PASSWORD", "storePassword")
val releaseSigningKeyAlias = signingValue("VIBE_ANDROID_KEY_ALIAS", "keyAlias")
val releaseSigningKeyPassword = signingValue("VIBE_ANDROID_KEY_PASSWORD", "keyPassword")
val hasReleaseSigning =
    !releaseSigningStoreFile.isNullOrEmpty() &&
        !releaseSigningStorePassword.isNullOrEmpty() &&
        !releaseSigningKeyAlias.isNullOrEmpty() &&
        !releaseSigningKeyPassword.isNullOrEmpty()

android {
    compileSdk = 36
    namespace = "org.fageac.vibeeverywhere"
    signingConfigs {
        create("release") {
            if (hasReleaseSigning) {
                storeFile = file(requireNotNull(releaseSigningStoreFile))
                storePassword = releaseSigningStorePassword
                keyAlias = releaseSigningKeyAlias
                keyPassword = releaseSigningKeyPassword
            }
        }
    }
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "true"
        applicationId = "org.fageac.vibeeverywhere"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {
                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            isMinifyEnabled = true
            if (hasReleaseSigning) {
                signingConfig = signingConfigs.getByName("release")
            }
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

if (hasReleaseSigning) {
    logger.lifecycle("Android release signing enabled for ${project.path}")
} else {
    logger.lifecycle("Android release signing not configured; release APK/AAB will be unsigned")
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")
