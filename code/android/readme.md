# Android

This directory contains the Android app wrapper for Chip DX.

The intended workflow is Android Studio first:

- Open `code/android` in Android Studio.
- Let Gradle sync.
- Install any SDK or NDK components Android Studio asks for.
- Build or run the `app` configuration.

That is the main supported way to get started from a clean checkout.

## You Need

- Android Studio, Android SDK, Android NDK.
  The NDK version is pinned in [app/build.gradle.kts](app/build.gradle.kts).
- Rust targets
  ```sh
  rustup target add aarch64-linux-android x86_64-linux-android
  ```
- Rust based tools
  ```sh
  cargo install cargo-ndk --locked
  cargo install paks --locked
  ```

If Android Studio prompts to create `local.properties`, let it do that.

## First Build

From a clean checkout:

1. Open `code/android` in Android Studio.
1. Wait for the Gradle sync to finish.
1. Build the app.

The Android build triggers two generated steps before packaging:

- It rebuilds the bundled data packs used by `chipjni`.
- It rebuilds the Rust JNI library and stages the generated `.so` files into the app build directory.

The debug APK ends up at:

```text
code/android/app/build/outputs/apk/debug/app-debug.apk
```

## CLI

If you want to build from a terminal, use the Gradle wrapper directly:

- Java is available to Gradle, either from `JAVA_HOME` or from `java` on `PATH`.
- the Android SDK path is known through `local.properties`, `ANDROID_HOME`, or `ANDROID_SDK_ROOT`.
- Cargo and Rust based tools are available on `PATH`.

Inside Android Studio, just use the normal build or run actions. Studio already drives the Gradle build and usually manages Java and SDK configuration for you.

On Linux or macOS:

```sh
./code/android/gradlew -p code/android :app:assembleDebug
```

On Windows:

```ps
code\\android\\gradlew.bat -p code/android :app:assembleDebug
```
