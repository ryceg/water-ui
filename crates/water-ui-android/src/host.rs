//! The platform-neutral half of the Android adapter: one `Engine` per process
//! owns the device and the compiled pipelines, each `Instance` forks them for
//! its own scene and renders into a caller-owned premultiplied sRGB buffer,
//! the layout of an `ARGB_8888` bitmap. Errors are `Code: detail`, the codes
//! the web adapter uses (`scene_tools::SceneToolError` for scenes, plus
//! `InitFailed`, `InstanceLimit`, `DeviceLost`, `Engine`), and the Android-only
//! `InvalidDetail`, `InvalidSurface`, `InvalidCurve`, `InvalidSize`. The JNI
//! layer adds `InvalidHandle`, `InvalidArgument`, `Poisoned` and `Panic`. A
//! device with no adapter is not an error: `Engine::create` returns `None`.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use water_ui_core::application::playback::DEFAULT_PAINT_WALL_FRACTION;
use water_ui_core::application::{EngineError, Playback, PlaybackState, ProgressCurve, Renderer};
use water_ui_core::domain::{Scene, Seed};
use water_ui_infra::authoring::DetailLevel;
use water_ui_infra::document::{parse_appended_json, parse_scene_json};
use water_ui_infra::export::linear_to_srgb;
use water_ui_infra::gpu::{GpuContext, GpuEngine};
use water_ui_wasm::scene_tools::{self, Surface};

/// As the web adapter: a phone has the memory of a browser tab, not of a
/// desktop, so the native default would be far too generous.
pub const DEFAULT_MAX_LIVE_INSTANCES: u32 = 4;

/// Per-instance checkpoint budget: ten checkpoints at the catalogue's 256^2 x
/// 4 pigments, the browser's figure, since several instances share one device.
pub const CHECKPOINT_BUDGET_BYTES: u64 = 48 * 1024 * 1024;

/// Below this alpha a pixel is written transparent; dividing colour by it
/// would only amplify quantisation noise (the PNG exporter's threshold).
const MIN_ALPHA: f32 = 1.0 / 1024.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostError(String);

impl HostError {
    pub fn new(code: &str, detail: impl std::fmt::Display) -> HostError {
        HostError(format!("{code}: {detail}"))
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for HostError {}

fn engine_err(e: EngineError) -> HostError {
    HostError::new("Engine", e)
}

/// A lost device crosses as `DeviceLost`, which the host answers by tearing
/// the engine down; a device error crosses as `Engine`, so only the instance
/// that hit it falls back.
fn guard_device(ctx: &GpuContext) -> Result<(), HostError> {
    if ctx.is_lost() {
        return Err(HostError::new("DeviceLost", "the GPU device was lost"));
    }
    if let Some(fault) = ctx.fault() {
        return Err(HostError::new("Engine", format!("device error: {fault}")));
    }
    Ok(())
}

/// How an instance spends its wall clock; see the web adapter's
/// `createInstance` for each field. `duration_ms` must be positive; zero keeps
/// the other fields' defaults.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Timing {
    pub duration_ms: f32,
    pub settle_fraction: f32,
    pub paint_wall_fraction: f32,
    pub checkpoint_budget_bytes: u64,
}

/// A catalogue artwork as `scene_tools::catalogue_scene_with_resolution`
/// takes it, with the strings still unparsed so the JNI layer stays thin.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogueRequest<'a> {
    pub artwork_id: &'a str,
    pub seed: u64,
    pub palette: &'a str,
    pub intensity: f32,
    pub detail: &'a str,
    pub surface: &'a str,
    /// 0 keeps the detail's default grid.
    pub sim_resolution: u32,
}

/// Scene JSON for a catalogue artwork, with no engine: a host composes its
/// own scenes (a bloom, say) on a catalogue donor's paper and parameters,
/// as the web package does with `catalogueScene`.
pub fn catalogue_scene_json(request: &CatalogueRequest<'_>) -> Result<String, HostError> {
    let (detail, surface) = parse_request(request)?;
    scene_tools::catalogue_scene_json_with_resolution(
        request.artwork_id,
        Seed(request.seed),
        request.palette,
        request.intensity,
        detail,
        surface,
        (request.sim_resolution > 0).then_some(request.sim_resolution),
    )
    .map_err(|e| HostError(e.to_string()))
}

fn parse_request(request: &CatalogueRequest<'_>) -> Result<(DetailLevel, Surface), HostError> {
    let detail = scene_tools::parse_detail(request.detail).ok_or_else(|| {
        HostError::new(
            "InvalidDetail",
            format!("{:?} is not small|medium|large|extralarge", request.detail),
        )
    })?;
    let surface = Surface::parse(request.surface).ok_or_else(|| {
        HostError::new(
            "InvalidSurface",
            format!("{:?} is not light|dark", request.surface),
        )
    })?;
    Ok((detail, surface))
}

/// `Send` but not `Sync` (the GPU timers hold `Cell`s): a host drives an
/// engine from one thread at a time, which the JNI layer enforces with a
/// mutex per handle.
pub struct Engine {
    ctx: GpuContext,
    template: GpuEngine,
    live: Arc<AtomicU32>,
    max_live: AtomicU32,
}

impl Engine {
    /// Opens a Vulkan device (Android's only backend with the storage-buffer
    /// limits the simulation needs) and compiles every pipeline once.
    /// `Ok(None)` when the device offers no usable adapter.
    pub fn create() -> Result<Option<Engine>, HostError> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
        if cfg!(target_os = "android") {
            descriptor.backends = wgpu::Backends::VULKAN;
        }
        let ctx = pollster::block_on(GpuContext::new_async(descriptor))
            .map_err(|e| HostError::new("InitFailed", e))?;
        let Some(ctx) = ctx else {
            return Ok(None);
        };
        Engine::with_context(ctx).map(Some)
    }

    pub fn with_context(ctx: GpuContext) -> Result<Engine, HostError> {
        let template = GpuEngine::new(ctx.clone())
            .map_err(|e| HostError::new("InitFailed", e))?
            .with_checkpoint_budget(CHECKPOINT_BUDGET_BYTES);
        Ok(Engine {
            ctx,
            template,
            live: Arc::new(AtomicU32::new(0)),
            max_live: AtomicU32::new(DEFAULT_MAX_LIVE_INSTANCES),
        })
    }

    pub fn adapter_name(&self) -> &str {
        self.ctx.adapter_name()
    }

    pub fn is_lost(&self) -> bool {
        self.ctx.is_lost()
    }

    pub fn is_software(&self) -> bool {
        self.ctx.is_software()
    }

    pub fn live_instances(&self) -> u32 {
        self.live.load(Ordering::Acquire)
    }

    pub fn max_live_instances(&self) -> u32 {
        self.max_live.load(Ordering::Acquire)
    }

    pub fn set_max_live_instances(&self, value: u32) {
        self.max_live.store(value.max(1), Ordering::Release);
    }

    pub fn catalogue_instance(
        &self,
        request: &CatalogueRequest<'_>,
        timing: Timing,
    ) -> Result<Instance, HostError> {
        self.admit()?;
        let (detail, surface) = parse_request(request)?;
        let scene = scene_tools::catalogue_scene_with_resolution(
            request.artwork_id,
            Seed(request.seed),
            request.palette,
            request.intensity,
            detail,
            surface,
            (request.sim_resolution > 0).then_some(request.sim_resolution),
        )
        .map_err(|e| HostError(e.to_string()))?;
        self.instance(scene, timing)
    }

    /// An instance for a versioned scene document, such as a bloom a host
    /// composes itself.
    pub fn scene_instance(&self, scene_json: &str, timing: Timing) -> Result<Instance, HostError> {
        self.admit()?;
        let scene = parse_scene_json(scene_json).map_err(|e| HostError::new("InvalidScene", e))?;
        self.instance(scene, timing)
    }

    fn admit(&self) -> Result<(), HostError> {
        guard_device(&self.ctx)?;
        let live = self.live_instances();
        if live >= self.max_live_instances() {
            return Err(HostError::new(
                "InstanceLimit",
                format!("{live} live instances already (maxLiveInstances)"),
            ));
        }
        Ok(())
    }

    fn instance(&self, mut scene: Scene, timing: Timing) -> Result<Instance, HostError> {
        if timing.settle_fraction.is_finite() && timing.settle_fraction > 0.0 {
            scene_tools::apply_settle_fraction(&mut scene, timing.settle_fraction);
        }
        let mut engine = self.template.fork();
        if timing.checkpoint_budget_bytes > 0 {
            engine = engine.with_checkpoint_budget(timing.checkpoint_budget_bytes);
        }
        let mut playback = Playback::new(engine, scene, timing.duration_ms)
            .map_err(|e| HostError::new("InvalidScene", e))?;
        if timing.paint_wall_fraction.is_finite() && timing.paint_wall_fraction > 0.0 {
            playback.set_progress_curve(ProgressCurve::reveal_for(
                playback.scene(),
                timing.paint_wall_fraction,
            ));
        }
        self.live.fetch_add(1, Ordering::AcqRel);
        Ok(Instance {
            playback,
            ctx: self.ctx.clone(),
            live: Arc::clone(&self.live),
        })
    }
}

pub struct Instance {
    playback: Playback<GpuEngine>,
    ctx: GpuContext,
    live: Arc<AtomicU32>,
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Instance {
    fn step(
        &mut self,
        f: impl FnOnce(&mut Playback<GpuEngine>) -> Result<(), EngineError>,
    ) -> Result<(), HostError> {
        guard_device(&self.ctx)?;
        f(&mut self.playback).map_err(engine_err)
    }

    /// `true` when the call moved the simulation, so the last render is stale.
    fn advance(
        &mut self,
        f: impl FnOnce(&mut Playback<GpuEngine>) -> Result<(), EngineError>,
    ) -> Result<bool, HostError> {
        let before = self.playback.current_tick();
        self.step(f)?;
        Ok(self.playback.current_tick() != before)
    }

    /// Normalised source window; output pixels and paper cover only it.
    pub fn set_crop(&mut self, x: f32, y: f32, width: f32, height: f32) -> Result<(), HostError> {
        self.playback
            .simulator()
            .set_crop([x, y, width, height])
            .map_err(engine_err)
    }

    pub fn sim_resolution(&self) -> u32 {
        self.playback.scene().sim_resolution.0
    }

    pub fn total_ticks(&self) -> u32 {
        self.playback.total_ticks()
    }

    pub fn current_tick(&self) -> u32 {
        self.playback.current_tick()
    }

    pub fn play(&mut self) {
        self.playback.play();
    }

    pub fn pause(&mut self) {
        self.playback.pause();
    }

    pub fn reset(&mut self) -> Result<(), HostError> {
        self.step(|p| p.reset())
    }

    pub fn advance_by_elapsed(&mut self, seconds: f32) -> Result<bool, HostError> {
        self.advance(|p| p.advance_by_elapsed(seconds))
    }

    pub fn advance_to_progress(&mut self, progress: f32) -> Result<bool, HostError> {
        self.advance(|p| p.advance_to_progress(progress))
    }

    /// Runs up to `ticks` more steps whatever the play state; `true` once
    /// finished and dry. Lets a host spread a still's run over several frames.
    pub fn advance_ticks(&mut self, ticks: u32) -> Result<bool, HostError> {
        self.step(|p| p.advance_ticks(ticks))?;
        Ok(self.is_finished())
    }

    pub fn seek_progress(&mut self, progress: f32) -> Result<(), HostError> {
        self.step(|p| p.seek_progress(progress))
    }

    pub fn finish_immediately(&mut self) -> Result<(), HostError> {
        self.step(|p| p.finish_immediately())
    }

    /// `frontLoaded`, `linear` or `reveal`, as the web adapter. A new
    /// instance starts on `reveal` at the default paint fraction.
    pub fn set_progress_curve(&mut self, curve: &str) -> Result<(), HostError> {
        let curve = match curve {
            "frontLoaded" | "front-loaded" => ProgressCurve::FrontLoaded,
            "linear" => ProgressCurve::Linear,
            "reveal" => {
                ProgressCurve::reveal_for(self.playback.scene(), DEFAULT_PAINT_WALL_FRACTION)
            }
            other => {
                return Err(HostError::new(
                    "InvalidCurve",
                    format!("{other:?} is not frontLoaded|linear|reveal"),
                ));
            }
        };
        self.playback.set_progress_curve(curve);
        Ok(())
    }

    /// See `Playback::go_live`.
    pub fn go_live(&mut self, ticks_per_second: f32, idle_ticks: u32) -> Result<(), HostError> {
        self.step(|p| p.go_live(ticks_per_second, idle_ticks))
    }

    /// A JSON array of `{ after_ticks, op }`; see `Playback::append`.
    pub fn append_operations(&mut self, json: &str) -> Result<(), HostError> {
        guard_device(&self.ctx)?;
        let events = parse_appended_json(json).map_err(|e| HostError::new("InvalidScene", e))?;
        self.playback.append(events).map_err(engine_err)
    }

    pub fn progress(&self) -> f32 {
        self.playback.progress()
    }

    pub fn is_finished(&self) -> bool {
        self.playback.state() == PlaybackState::Finished
    }

    pub fn is_playing(&self) -> bool {
        self.playback.state() == PlaybackState::Playing
    }

    /// Renders the current state at `width` x `height` into `out`, row-major
    /// premultiplied sRGB RGBA8: the memory layout of an `ARGB_8888` bitmap,
    /// so the host copies it in with `copyPixelsFromBuffer`.
    pub fn render_into(
        &mut self,
        width: u32,
        height: u32,
        out: &mut [u8],
    ) -> Result<(), HostError> {
        // Checked: a 32-bit ABI overflows at a few hundred megapixels.
        let needed = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(4))
            .unwrap_or(usize::MAX);
        if width == 0 || height == 0 || out.len() < needed {
            return Err(HostError::new(
                "InvalidSize",
                format!(
                    "{width}x{height} needs {needed} bytes, the buffer holds {}",
                    out.len()
                ),
            ));
        }
        guard_device(&self.ctx)?;
        let image = self
            .playback
            .simulator()
            .render(width, height)
            .map_err(engine_err)?;
        premultiplied_srgb8(&image.rgba, &mut out[..needed]);
        Ok(())
    }
}

/// Premultiplied linear RGBA (the engine's frame) to premultiplied sRGB
/// RGBA8 (an Android bitmap): colour is divided out of alpha, encoded, then
/// multiplied back in gamma space, which is how the platform composites.
pub fn premultiplied_srgb8(linear: &[f32], out: &mut [u8]) {
    for (px, dst) in linear
        .as_chunks::<4>()
        .0
        .iter()
        .zip(out.as_chunks_mut::<4>().0)
    {
        let a = px[3].clamp(0.0, 1.0);
        if a < MIN_ALPHA {
            *dst = [0, 0, 0, 0];
            continue;
        }
        for c in 0..3 {
            let straight = (px[c] / a).clamp(0.0, 1.0);
            dst[c] = (linear_to_srgb(straight) * a * 255.0 + 0.5) as u8;
        }
        dst[3] = (a * 255.0 + 0.5) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_pixels_encode_to_srgb() {
        let mut out = [0u8; 8];
        premultiplied_srgb8(&[1.0, 0.0, 0.215_861, 1.0, 0.0, 0.0, 0.0, 1.0], &mut out);
        assert_eq!(out, [255, 0, 128, 255, 0, 0, 0, 255]);
    }

    #[test]
    fn translucent_pixels_stay_premultiplied_in_gamma_space() {
        // Half-covered linear mid-grey: straight 0.214 is sRGB ~0.5, so the
        // premultiplied byte is ~0.5 * 0.5 * 255.
        let mut out = [0u8; 4];
        premultiplied_srgb8(&[0.107_02, 0.107_02, 0.107_02, 0.5], &mut out);
        assert_eq!(out, [64, 64, 64, 128]);
    }

    #[test]
    fn near_transparent_pixels_are_cleared() {
        let mut out = [9u8; 4];
        premultiplied_srgb8(&[0.0001, 0.0, 0.0, 0.0002], &mut out);
        assert_eq!(out, [0, 0, 0, 0]);
    }

    #[test]
    fn errors_carry_the_web_adapter_codes() {
        assert_eq!(
            HostError::new("InstanceLimit", "4 live").message(),
            "InstanceLimit: 4 live"
        );
    }
}
