# Keep rules for a host that minifies (R8/ProGuard): libwater_ui_android.so
# binds WaterUiNative's methods by their Java names and throws
# WaterUiException through its (String) constructor, which no Kotlin code
# calls. Add this file to the host module's consumerProguardFiles.
-keep class io.github.ryceg.waterui.WaterUiNative { *; }
-keep class io.github.ryceg.waterui.WaterUiException { <init>(java.lang.String); }
