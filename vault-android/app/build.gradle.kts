import java.io.ByteArrayOutputStream
import com.android.build.gradle.internal.cxx.configure.gradleLocalProperties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("kotlin-kapt")
    id("com.google.dagger.hilt.android")
}

val localProperties = gradleLocalProperties(rootDir)

android {
    namespace = "net.koofr.vault"
    compileSdk = 34

    defaultConfig {
        applicationId = "net.koofr.vault"
        minSdk = 23
        targetSdk = 34
        versionCode = 116001
        versionName = "0.1.16"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }

        val filesAuthorityValue = "$applicationId.files"
        manifestPlaceholders["filesAuthority"] = filesAuthorityValue
        buildConfigField(
            "String",
            "FILES_AUTHORITY",
            "\"$filesAuthorityValue\""
        )

        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86", "x86_64")
        }
    }

    if (!localProperties.getProperty("signingConfigs.release.storeFile").isNullOrEmpty()) {
        signingConfigs {
            create("release") {
                storeFile =
                    file(localProperties.getProperty("signingConfigs.release.storeFile"))
                storePassword =
                    localProperties.getProperty("signingConfigs.release.storePassword")
                keyAlias = localProperties.getProperty("signingConfigs.release.keyAlias")
                keyPassword =
                    localProperties.getProperty("signingConfigs.release.keyPassword")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfigs.findByName("release")?.let {
                signingConfig = it
            }
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.3"
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    ndkVersion = localProperties.getProperty("android.ndkVersion")
    sourceSets {
        getByName("debug") {
            jniLibs.srcDir(file("$buildDir/rustJniLibs/android"))
        }
        getByName("release") {
            jniLibs.srcDir(file("$buildDir/rustJniLibs/android"))
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.6.2")
    implementation("androidx.activity:activity-compose:1.7.2")
    implementation(platform("androidx.compose:compose-bom:2023.10.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.browser:browser:1.6.0")
    implementation("androidx.security:security-crypto:1.1.0-alpha06")
    implementation("androidx.navigation:navigation-compose:2.7.4")
    implementation("net.java.dev.jna:jna:5.13.0@aar")
    implementation("com.google.dagger:hilt-android:2.46.1")
    implementation("androidx.hilt:hilt-navigation-compose:1.1.0-beta01")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("androidx.biometric:biometric:1.1.0")
    implementation("com.google.accompanist:accompanist-permissions:0.30.1")
    implementation("io.coil-kt:coil-compose-base:2.4.0")
    implementation("io.coil-kt:coil-gif:2.4.0")
    implementation("io.coil-kt:coil-svg:2.4.0")
    implementation("com.github.chrisbanes:PhotoView:565505d5cb")
//    implementation("com.davemorrissey.labs:subsampling-scale-image-view-androidx:3.10.0")
    implementation("androidx.media3:media3-exoplayer:1.1.1")
    implementation("androidx.media3:media3-ui:1.1.1")
    kapt("com.google.dagger:hilt-compiler:2.48")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
    androidTestImplementation(platform("androidx.compose:compose-bom:2023.10.00"))
    androidTestImplementation("androidx.compose.ui:ui-test-junit4")
    androidTestImplementation("androidx.test.uiautomator:uiautomator:2.3.0-alpha04")
    debugImplementation("androidx.compose.ui:ui-tooling")
    debugImplementation("androidx.compose.ui:ui-test-manifest")
}

kapt {
    correctErrorTypes = true
}

val uniFFIBindingsDir = "${buildDir}/generated/source/uniffi/java"

task<Exec>("generateUniFFIBindings") {
    inputs.file("${project.projectDir}/../../vault-mobile/src/vault-mobile.udl")
    outputs.dir(uniFFIBindingsDir)

    workingDir = file("${project.projectDir}/../../vault-mobile/uniffi-bindgen")
    commandLine(
        "cargo",
        "run",
        "generate",
        "../src/vault-mobile.udl",
        "--language",
        "kotlin",
        "--out-dir",
        uniFFIBindingsDir
    )

    doLast {
        println("UniFFI bindings generated successfully!")
    }
}

kotlin {
    sourceSets {
        main {
            kotlin.srcDir(uniFFIBindingsDir)
        }
    }
}

apply(plugin = "org.mozilla.rust-android-gradle.rust-android")

fun getGitRevision(): String {
    val stdout = ByteArrayOutputStream()
    project.exec {
        commandLine("git", "rev-parse", "--short", "HEAD")
        standardOutput = stdout
    }
    return String(stdout.toByteArray()).trim()
}

fun getGitRelease(): String {
    val stdout = ByteArrayOutputStream()
    project.exec {
        commandLine("git", "describe", "--tags", "--exact-match")
        standardOutput = stdout
        isIgnoreExitValue = true
    }
    return String(stdout.toByteArray()).trim()
}

extensions.configure(com.nishtahir.CargoExtension::class) {
    module = "../../vault-mobile"
    libname = "vault_mobile"
    targets = listOf("arm", "arm64", "x86", "x86_64")
//    targets = listOf("x86")
    targetDirectory = "../../target"
    pythonCommand = "python3"
    profile = System.getenv("GRADLE_CARGO_PROFILE") ?: "release"
    exec = { spec, _ ->
        spec.environment("GIT_REVISION", getGitRevision())
        spec.environment("GIT_RELEASE", getGitRelease())
    }
}

//tasks.whenTaskAdded {
//    if (name == "javaPreCompileDebug" || name == "javaPreCompileRelease") {
//        dependsOn("cargoBuild")
//        dependsOn("generateUniFFIBindings")
//    }
//    if (name == "kaptGenerateStubsDebugKotlin" || name == "kaptGenerateStubsReleaseKotlin") {
//        dependsOn("generateUniFFIBindings")
//    }
//}

// mergeDebugNativeLibs and mergeReleaseNativeLibs don't update the .so files in
// build/intermediates/merged_jni_libs. if we manually delete this folder before
// cargoBuild the new libraries will be copied correctly without needing to run
// clean task
task<Delete>("cleanupMergedJniLibs") {
    delete(file("${buildDir}/intermediates/merged_jni_libs"))

    doLast {
        println("Deleted '${buildDir}/intermediates/merged_jni_libs'")
    }
}

tasks.whenTaskAdded {
    if (name == "cargoBuild") {
        dependsOn("cleanupMergedJniLibs")
    }
}

task("printJniLibs") {
    doLast {
        println("debug")
        println(android.sourceSets["debug"].jniLibs)
        println("release")
        println(android.sourceSets["release"].jniLibs)
    }
}
