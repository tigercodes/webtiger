// WebTiger for Android (003): one window over the Rust core, with the System WebView as its
// engine. The app decides nothing about browsing: it asks the core (:core-engine) and draws.
plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "com.tigercodes.webtiger"
    compileSdk = libs.versions.compile.sdk.get().toInt()
    buildToolsVersion = libs.versions.build.tools.get()
    // The core's NDK: the app strips the native libraries with it when it packs them.
    ndkVersion = libs.versions.ndk.get()
    defaultConfig {
        applicationId = "com.tigercodes.webtiger"
        minSdk = libs.versions.min.sdk.get().toInt()
        targetSdk = libs.versions.compile.sdk.get().toInt()
        versionCode = 1
        versionName = "0.1.0"
        // The tests in src/androidTest run on a device or an emulator, inside the app's process.
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        // The ABIs the core is built for. Without this a library's own 32-bit builds would put
        // the app on 32-bit devices, where the core has none.
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }
    buildFeatures {
        compose = true
    }
    testOptions {
        unitTests {
            // The window's tests draw it on Robolectric's Android, with the app's own resources.
            isIncludeAndroidResources = true
            all { test ->
                test.testLogging {
                    events("failed")
                    exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
                }
                // Robolectric's Android 16 reaches into the JDK for its file descriptors.
                test.jvmArgs("--add-opens=java.base/jdk.internal.access=ALL-UNNAMED")
            }
        }
    }
}

dependencies {
    implementation(project(":core-engine"))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.webkit)
    implementation(libs.kotlinx.coroutines.android)
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.ui)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.robolectric)
    testImplementation(libs.androidx.test.core)
    testImplementation(libs.compose.ui.test.junit4)
    // The empty activity Compose's test rule draws in.
    debugImplementation(libs.compose.ui.test.manifest)

    // On a device: the adapter suite against the real WebView, and the window on a screen.
    androidTestImplementation(libs.junit)
    androidTestImplementation(libs.androidx.test.core)
    androidTestImplementation(libs.androidx.test.runner)
    androidTestImplementation(libs.androidx.test.ext.junit)
    androidTestImplementation(platform(libs.compose.bom))
    androidTestImplementation(libs.compose.ui.test.junit4)
}

// The window is tested over the real core too (ThroughTheCoreTest): the library :core-engine
// builds for this machine, as its own tests load it.
tasks.withType<Test>().configureEach {
    // The design catalogue's pictures (004): where they are checked in, and where what was drawn
    // goes when it does not match.
    val snapshots = layout.projectDirectory.dir("src/test/snapshots")
    systemProperty("webtiger.snapshots", snapshots.asFile.absolutePath)
    systemProperty("webtiger.snapshots.failed", layout.buildDirectory.dir("snapshots").get().asFile.absolutePath)
    inputs.dir(snapshots).withPropertyName("snapshots").withPathSensitivity(PathSensitivity.RELATIVE).optional()
    // Recording them is asked for in the environment, and must not be answered from the cache.
    inputs.property("recordSnapshots", providers.environmentVariable("WEBTIGER_RECORD_SNAPSHOTS").orElse(""))
    dependsOn(":core-engine:cargoBuildHost")
    val libraries = rootProject.layout.projectDirectory.dir("../Core/target/jvm/debug")
    systemProperty("java.library.path", libraries.asFile.absolutePath)
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    inputs.file(libraries.file(System.mapLibraryName("webtiger_core")))
        .withPropertyName("coreLibrary")
        .withPathSensitivity(PathSensitivity.NONE)
}
