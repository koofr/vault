import java.io.ByteArrayOutputStream
import com.android.build.gradle.internal.cxx.configure.gradleLocalProperties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.ksp)
    alias(libs.plugins.hilt.android)
    alias(libs.plugins.compose.compiler)
}

val localProperties = gradleLocalProperties(rootDir, providers)

android {
    namespace = "net.koofr.vault"
    compileSdk = 35

    defaultConfig {
        applicationId = "net.koofr.vault"
        minSdk = 23
        targetSdk = 35
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
            jniLibs.srcDir(layout.buildDirectory.dir("rustJniLibs/android"))
        }
        getByName("release") {
            jniLibs.srcDir(layout.buildDirectory.dir("rustJniLibs/android"))
        }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)

    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.material.icons)

    implementation(libs.androidx.browser)
    implementation(libs.androidx.security.crypto)
    implementation(libs.androidx.navigation.compose)
    implementation(variantOf(libs.jna) { artifactType("aar") })

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)
    implementation(libs.androidx.hilt.navigation.compose)

    implementation(libs.androidx.appcompat)

    implementation(libs.androidx.biometric)

    implementation(libs.accompanist.permissions)

    implementation(libs.coil.compose.base)
    implementation(libs.coil.gif)
    implementation(libs.coil.svg)

    implementation(libs.photoview)

    implementation(libs.androidx.media3.exoplayer)
    implementation(libs.androidx.media3.ui)

    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    androidTestImplementation(libs.androidx.uiautomator)

    debugImplementation(libs.androidx.compose.ui.tooling)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}

val uniFFIBindingsDir = layout.buildDirectory.dir("generated/source/uniffi/java")

tasks.register<Exec>("generateUniFFIBindings") {
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
        uniFFIBindingsDir.get().asFile
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

val mergedJniLibsDir = layout.buildDirectory.dir("intermediates/merged_jni_libs")

// mergeDebugNativeLibs and mergeReleaseNativeLibs don't update the .so files in
// build/intermediates/merged_jni_libs. if we manually delete this folder before
// cargoBuild the new libraries will be copied correctly without needing to run
// clean task
tasks.register<Delete>("cleanupMergedJniLibs") {
    delete(mergedJniLibsDir)

    doLast {
        println("Deleted '${mergedJniLibsDir.get().asFile}'")
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
