//! JNI surface over `host`, bound to `io.github.ryceg.waterui.WaterUiNative`
//! (see `kotlin/` for the Kotlin side; the two change together). Engines and
//! instances cross as opaque `jlong` handles the Kotlin wrapper owns and
//! frees exactly once. Every failure crosses as a `WaterUiException` whose
//! message is `Code: detail`; a panic crosses the same way as `Panic`, so it
//! never unwinds into the JVM.

use std::panic::{AssertUnwindSafe, catch_unwind};

use jni::JNIEnv;
use jni::objects::{JByteBuffer, JClass, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jfloat, jint, jlong, jstring};

use crate::host::{self, CatalogueRequest, Engine, HostError, Instance, Timing};

const EXCEPTION_CLASS: &str = "io/github/ryceg/waterui/WaterUiException";

/// Runs `f`, turning an error or a panic into a pending Java exception and
/// `fallback` as the (ignored) return value.
fn call<T>(
    env: &mut JNIEnv,
    fallback: T,
    f: impl FnOnce(&mut JNIEnv) -> Result<T, HostError>,
) -> T {
    let outcome = catch_unwind(AssertUnwindSafe(|| f(env)));
    let message = match outcome {
        Ok(Ok(value)) => return value,
        Ok(Err(e)) => e.to_string(),
        Err(panic) => {
            let detail = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_string());
            format!("Panic: {detail}")
        }
    };
    if !env.exception_check().unwrap_or(false) {
        let _ = env.throw_new(EXCEPTION_CLASS, message);
    }
    fallback
}

fn string(env: &mut JNIEnv, value: &JString) -> Result<String, HostError> {
    env.get_string(value)
        .map(Into::into)
        .map_err(|e| HostError::new("InvalidArgument", e))
}

fn flag(value: bool) -> jboolean {
    if value { JNI_TRUE } else { JNI_FALSE }
}

/// # Safety
/// `handle` is a live `Engine` handle from `engineCreate`.
unsafe fn engine<'a>(handle: jlong) -> Result<&'a Engine, HostError> {
    if handle == 0 {
        return Err(HostError::new("InvalidHandle", "engine handle is 0"));
    }
    Ok(unsafe { &*(handle as *const Engine) })
}

/// # Safety
/// `handle` is a live `Instance` handle from an `instanceCreate*`, used by
/// one thread at a time.
unsafe fn instance<'a>(handle: jlong) -> Result<&'a mut Instance, HostError> {
    if handle == 0 {
        return Err(HostError::new("InvalidHandle", "instance handle is 0"));
    }
    Ok(unsafe { &mut *(handle as *mut Instance) })
}

/// `checkpoint_budget_bytes` 0 keeps the engine's budget; any value below
/// one checkpoint (1, say) keeps none, for an instance released on finish.
fn timing(
    duration_ms: jfloat,
    settle_fraction: jfloat,
    paint_wall_fraction: jfloat,
    checkpoint_budget_bytes: jlong,
) -> Timing {
    Timing {
        duration_ms,
        settle_fraction,
        paint_wall_fraction,
        checkpoint_budget_bytes: checkpoint_budget_bytes.max(0) as u64,
    }
}

/// 0 when the device has no usable Vulkan adapter; throws `InitFailed` when
/// it has one that would not open or compile.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_engineCreate(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    call(&mut env, 0, |_| {
        Ok(Engine::create()?.map_or(0, |e| Box::into_raw(Box::new(e)) as jlong))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_engineDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        drop(unsafe { Box::from_raw(handle as *mut Engine) });
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_engineAdapterName(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    call(&mut env, std::ptr::null_mut(), |env| {
        let name = unsafe { engine(handle) }?.adapter_name().to_string();
        env.new_string(name)
            .map(|s| s.into_raw())
            .map_err(|e| HostError::new("Engine", e))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_engineIsLost(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    call(&mut env, JNI_TRUE, |_| {
        Ok(flag(unsafe { engine(handle) }?.is_lost()))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_engineIsSoftware(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    call(&mut env, JNI_TRUE, |_| {
        Ok(flag(unsafe { engine(handle) }?.is_software()))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_engineSetMaxLiveInstances(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    value: jint,
) {
    call(&mut env, (), |_| {
        unsafe { engine(handle) }?.set_max_live_instances(value.max(1) as u32);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_engineLiveInstances(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    call(&mut env, 0, |_| {
        Ok(unsafe { engine(handle) }?.live_instances() as jint)
    })
}

#[allow(clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceCreateCatalogue(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    artwork_id: JString,
    seed: jlong,
    palette: JString,
    intensity: jfloat,
    detail: JString,
    surface: JString,
    sim_resolution: jint,
    duration_ms: jfloat,
    settle_fraction: jfloat,
    paint_wall_fraction: jfloat,
    checkpoint_budget_bytes: jlong,
) -> jlong {
    call(&mut env, 0, |env| {
        let engine = unsafe { engine(handle) }?;
        let artwork_id = string(env, &artwork_id)?;
        let palette = string(env, &palette)?;
        let detail = string(env, &detail)?;
        let surface = string(env, &surface)?;
        let request = CatalogueRequest {
            artwork_id: &artwork_id,
            seed: seed.max(0) as u64,
            palette: &palette,
            intensity,
            detail: &detail,
            surface: &surface,
            sim_resolution: sim_resolution.max(0) as u32,
        };
        let instance = engine.catalogue_instance(
            &request,
            timing(
                duration_ms,
                settle_fraction,
                paint_wall_fraction,
                checkpoint_budget_bytes,
            ),
        )?;
        Ok(Box::into_raw(Box::new(instance)) as jlong)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceCreateScene(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    scene_json: JString,
    duration_ms: jfloat,
    settle_fraction: jfloat,
    paint_wall_fraction: jfloat,
    checkpoint_budget_bytes: jlong,
) -> jlong {
    call(&mut env, 0, |env| {
        let engine = unsafe { engine(handle) }?;
        let json = string(env, &scene_json)?;
        let instance = engine.scene_instance(
            &json,
            timing(
                duration_ms,
                settle_fraction,
                paint_wall_fraction,
                checkpoint_budget_bytes,
            ),
        )?;
        Ok(Box::into_raw(Box::new(instance)) as jlong)
    })
}

/// Scene JSON for a catalogue artwork; needs no engine (see
/// `host::catalogue_scene_json`).
#[allow(clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_catalogueScene(
    mut env: JNIEnv,
    _class: JClass,
    artwork_id: JString,
    seed: jlong,
    palette: JString,
    intensity: jfloat,
    detail: JString,
    surface: JString,
    sim_resolution: jint,
) -> jstring {
    call(&mut env, std::ptr::null_mut(), |env| {
        let artwork_id = string(env, &artwork_id)?;
        let palette = string(env, &palette)?;
        let detail = string(env, &detail)?;
        let surface = string(env, &surface)?;
        let json = host::catalogue_scene_json(&CatalogueRequest {
            artwork_id: &artwork_id,
            seed: seed.max(0) as u64,
            palette: &palette,
            intensity,
            detail: &detail,
            surface: &surface,
            sim_resolution: sim_resolution.max(0) as u32,
        })?;
        env.new_string(json)
            .map(|s| s.into_raw())
            .map_err(|e| HostError::new("Engine", e))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        drop(unsafe { Box::from_raw(handle as *mut Instance) });
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceSetCrop(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) {
    call(&mut env, (), |_| {
        unsafe { instance(handle) }?.set_crop(x, y, width, height)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceSimResolution(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    call(&mut env, 0, |_| {
        Ok(unsafe { instance(handle) }?.sim_resolution() as jint)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceTotalTicks(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    call(&mut env, 0, |_| {
        Ok(unsafe { instance(handle) }?.total_ticks() as jint)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instancePlay(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    call(&mut env, (), |_| {
        unsafe { instance(handle) }?.play();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instancePause(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    call(&mut env, (), |_| {
        unsafe { instance(handle) }?.pause();
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceReset(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    call(&mut env, (), |_| unsafe { instance(handle) }?.reset())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceAdvanceByElapsed(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    seconds: jfloat,
) -> jboolean {
    call(&mut env, JNI_FALSE, |_| {
        Ok(flag(
            unsafe { instance(handle) }?.advance_by_elapsed(seconds)?,
        ))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceAdvanceToProgress(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    progress: jfloat,
) -> jboolean {
    call(&mut env, JNI_FALSE, |_| {
        Ok(flag(
            unsafe { instance(handle) }?.advance_to_progress(progress)?,
        ))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceAdvanceTicks(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    ticks: jint,
) -> jboolean {
    call(&mut env, JNI_FALSE, |_| {
        Ok(flag(
            unsafe { instance(handle) }?.advance_ticks(ticks.max(0) as u32)?,
        ))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceSeekProgress(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    progress: jfloat,
) {
    call(&mut env, (), |_| {
        unsafe { instance(handle) }?.seek_progress(progress)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceFinishImmediately(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    call(&mut env, (), |_| {
        unsafe { instance(handle) }?.finish_immediately()
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceSetProgressCurve(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    curve: JString,
) {
    call(&mut env, (), |env| {
        let curve = string(env, &curve)?;
        unsafe { instance(handle) }?.set_progress_curve(&curve)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceGoLive(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    ticks_per_second: jfloat,
    idle_ticks: jint,
) {
    call(&mut env, (), |_| {
        unsafe { instance(handle) }?.go_live(ticks_per_second, idle_ticks.max(0) as u32)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceAppendOperations(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    json: JString,
) {
    call(&mut env, (), |env| {
        let json = string(env, &json)?;
        unsafe { instance(handle) }?.append_operations(&json)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceProgress(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jfloat {
    call(&mut env, 0.0, |_| {
        Ok(unsafe { instance(handle) }?.progress())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceIsFinished(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    call(&mut env, JNI_TRUE, |_| {
        Ok(flag(unsafe { instance(handle) }?.is_finished()))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceIsPlaying(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    call(&mut env, JNI_FALSE, |_| {
        Ok(flag(unsafe { instance(handle) }?.is_playing()))
    })
}

/// Renders into a direct `ByteBuffer` of at least `width * height * 4`
/// bytes (premultiplied sRGB RGBA8; see `Instance::render_into`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_ryceg_waterui_WaterUiNative_instanceRender(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
    buffer: JByteBuffer,
) {
    call(&mut env, (), |env| {
        let address = env
            .get_direct_buffer_address(&buffer)
            .map_err(|e| HostError::new("InvalidArgument", format!("not a direct buffer: {e}")))?;
        let capacity = env
            .get_direct_buffer_capacity(&buffer)
            .map_err(|e| HostError::new("InvalidArgument", e))?;
        // SAFETY: the JVM keeps a direct buffer's memory alive and fixed for
        // the buffer's lifetime, which spans this call.
        let out = unsafe { std::slice::from_raw_parts_mut(address, capacity) };
        unsafe { instance(handle) }?.render_into(width.max(0) as u32, height.max(0) as u32, out)
    })
}
