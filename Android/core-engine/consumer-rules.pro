# The core hands each envelope to EnvelopeListener.onEnvelope(byte[]), which its JNI glue finds
# by name (Core/crates/wt-ffi/src/android.rs). Renamed or removed, the app would hear nothing
# from the core. CoreLibrary's native methods keep their names through Android's default rules.
-keep interface com.tigercodes.webtiger.core.EnvelopeListener { void onEnvelope(byte[]); }
-keepclassmembers class * implements com.tigercodes.webtiger.core.EnvelopeListener { void onEnvelope(byte[]); }
