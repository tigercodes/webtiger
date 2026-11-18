// WebTiger for Android (docs/features/003-android-shell.md): the app, and the core's client
// over the Rust core (Core/, which :core-engine builds for Android).
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "WebTiger"

include(":app", ":core-engine")
