package io.github.ryceg.waterui

import java.nio.ByteBuffer

/**
 * The JNI surface of `libwater_ui_android.so` (`crates/water-ui-android`).
 * This file and `src/jni_api.rs` are one contract: a renamed or retyped
 * function here must change there too.
 *
 * Engines and instances are opaque handles. The caller frees each exactly once
 * (`engineDestroy`, `instanceDestroy`) and drives an engine and its instances
 * from one thread: the engine is `Send` but not `Sync`. Every failure throws
 * [WaterUiException]; its [WaterUiException.code] is one of the web adapter's
 * codes (`GpuUnavailable`, `InitFailed`, `InstanceLimit`, `DeviceLost`,
 * `Engine`, `UnknownArtwork`, `UnknownPalette`, `InvalidScene`, ...).
 */
object WaterUiNative {
    const val LIBRARY = "water_ui_android"

    /** Loads the library; false when this build or ABI ships none. */
    fun load(): Boolean = try {
        System.loadLibrary(LIBRARY)
        true
    } catch (_: UnsatisfiedLinkError) {
        false
    }

    /** 0 when the device has no usable Vulkan adapter. */
    @JvmStatic external fun engineCreate(): Long

    @JvmStatic external fun engineDestroy(engine: Long)

    @JvmStatic external fun engineAdapterName(engine: Long): String

    @JvmStatic external fun engineIsLost(engine: Long): Boolean

    /** A software rasteriser (SwiftShader on an emulator): correct but slow. */
    @JvmStatic external fun engineIsSoftware(engine: Long): Boolean

    @JvmStatic external fun engineSetMaxLiveInstances(engine: Long, value: Int)

    @JvmStatic external fun engineLiveInstances(engine: Long): Int

    /**
     * A catalogue artwork. `detail` is `small|medium|large|extralarge`,
     * `surface` `light|dark`; `simResolution` 0 keeps the detail's grid.
     * `paintWallFraction` is the share of `durationMs` the brushwork gets
     * (1 - tail); `checkpointBudgetBytes` 0 keeps the default, 1 keeps none.
     */
    @JvmStatic external fun instanceCreateCatalogue(
        engine: Long,
        artworkId: String,
        seed: Long,
        palette: String,
        intensity: Float,
        detail: String,
        surface: String,
        simResolution: Int,
        durationMs: Float,
        settleFraction: Float,
        paintWallFraction: Float,
        checkpointBudgetBytes: Long,
    ): Long

    /**
     * The scene document a catalogue artwork builds, with no engine: the
     * donor a host-composed scene (a bloom) takes its paper and parameters
     * from. Arguments as [instanceCreateCatalogue].
     */
    @JvmStatic external fun catalogueScene(
        artworkId: String,
        seed: Long,
        palette: String,
        intensity: Float,
        detail: String,
        surface: String,
        simResolution: Int,
    ): String

    /** A versioned scene document (see docs/scene-format.md). */
    @JvmStatic external fun instanceCreateScene(
        engine: Long,
        sceneJson: String,
        durationMs: Float,
        settleFraction: Float,
        paintWallFraction: Float,
        checkpointBudgetBytes: Long,
    ): Long

    @JvmStatic external fun instanceDestroy(instance: Long)

    @JvmStatic external fun instanceSetCrop(instance: Long, x: Float, y: Float, width: Float, height: Float)

    @JvmStatic external fun instanceSimResolution(instance: Long): Int

    @JvmStatic external fun instanceTotalTicks(instance: Long): Int

    @JvmStatic external fun instancePlay(instance: Long)

    @JvmStatic external fun instancePause(instance: Long)

    @JvmStatic external fun instanceReset(instance: Long)

    /** True when the simulation moved, so the last render is stale. */
    @JvmStatic external fun instanceAdvanceByElapsed(instance: Long, seconds: Float): Boolean

    @JvmStatic external fun instanceAdvanceToProgress(instance: Long, progress: Float): Boolean

    /** Runs up to `ticks` more steps; true once finished and dry. */
    @JvmStatic external fun instanceAdvanceTicks(instance: Long, ticks: Int): Boolean

    @JvmStatic external fun instanceSeekProgress(instance: Long, progress: Float)

    @JvmStatic external fun instanceFinishImmediately(instance: Long)

    /** `frontLoaded`, `linear` or `reveal`. */
    @JvmStatic external fun instanceSetProgressCurve(instance: Long, curve: String)

    @JvmStatic external fun instanceGoLive(instance: Long, ticksPerSecond: Float, idleTicks: Int)

    /** A JSON array of `{ after_ticks, op }`. */
    @JvmStatic external fun instanceAppendOperations(instance: Long, json: String)

    @JvmStatic external fun instanceProgress(instance: Long): Float

    @JvmStatic external fun instanceIsFinished(instance: Long): Boolean

    @JvmStatic external fun instanceIsPlaying(instance: Long): Boolean

    /**
     * Renders the current state into a direct [buffer] of at least
     * `width * height * 4` bytes, premultiplied sRGB RGBA8: an `ARGB_8888`
     * bitmap's layout, so `Bitmap.copyPixelsFromBuffer` takes it as is.
     */
    @JvmStatic external fun instanceRender(instance: Long, width: Int, height: Int, buffer: ByteBuffer)
}

/** A native failure; [message] is `Code: detail`. */
class WaterUiException(message: String) : RuntimeException(message) {
    /** The leading code, e.g. `InstanceLimit`. */
    val code: String get() = message.orEmpty().substringBefore(':')
}
