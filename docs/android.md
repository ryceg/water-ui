# Android adapter

`crates/water-ui-android` runs the same engine natively on Android, through
Vulkan, the way `water-ui-wasm` runs it in a browser through WebGPU. It is a
thin adapter. The device, instances and pixel conversion live in `host`, a
platform-neutral module the host-side tests drive on a desktop GPU. The JNI
functions in `jni_api` only marshal arguments.

## Surface

The Kotlin binding is `kotlin/io/github/ryceg/waterui/WaterUiNative.kt`. It is
part of the contract, so a host app compiles it from this checkout instead of
copying it.

| Rust (`host`) | Kotlin (`WaterUiNative`) | Web equivalent |
|---|---|---|
| `Engine::create` | `engineCreate` (0 = no adapter) | `WatercolourEngine.create` |
| `Engine::catalogue_instance` | `instanceCreateCatalogue` | `createCatalogueInstance` |
| `Engine::scene_instance` | `instanceCreateScene` | `createInstance` |
| `Instance::render_into` | `instanceRender` into a direct `ByteBuffer` | `render` to a canvas |
| `host::catalogue_scene_json` | `catalogueScene` (no engine needed) | `catalogueScene` |
| play, pause, reset, advance, seek, finish, curve, go live, append | same names, `instance` prefix | same |

These behave as on the web:

- Errors are `Code: detail` and cross as `WaterUiException`. The codes are the
  same ones the web uses: `InstanceLimit`, `DeviceLost`, `UnknownArtwork`, and
  so on. On top of those:
  - Android adds `InvalidSize` and `InvalidHandle` (a zero handle).
  - It adds `InvalidArgument` (for example, a buffer that isn't direct).
  - It adds `Poisoned`: an earlier panic left the handle unusable, so destroy it.
  - A device with no Vulkan adapter is not an error: `engineCreate` returns 0.
- At most four instances are live at once. A host can change that with
  `engineSetMaxLiveInstances`.
- Each instance's checkpoint budget is 48 MB, the browser's figure. Passing a
  budget of 1 byte keeps no checkpoints, which suits an instance released
  when its reveal finishes.
- Panics are caught at the JNI boundary and thrown as `Panic`, so they never
  unwind into the JVM.

Each handle points to a mutex. A call from a second thread waits for the
first call to finish, so it never races it. A single-threaded executor still
keeps the GPU's work in order.

Free each handle exactly once. A freed handle is a dangling pointer the
adapter cannot detect, so wrap handles in a class that zeroes its handle on
close.

A host that minifies must keep the binding. Add
`kotlin/consumer-rules.pro` to the module's `consumerProguardFiles`. Without
it, R8 strips the exception constructor the native side calls.

Seeds are the full 64 bits of a `Long`, and negative values are valid seeds.

## Pixels

`instanceRender` blocks on GPU readback and writes **premultiplied sRGB
RGBA8**. That is the in-memory layout of an `ARGB_8888` `Bitmap`, so the host
calls `Bitmap.copyPixelsFromBuffer` and draws the result like any other image.
The adapter writes from the buffer's start and ignores its position, but
`copyPixelsFromBuffer` reads from the position and advances it, so rewind the
buffer before each copy.
The engine's own frames are premultiplied *linear*. The adapter divides alpha
out, encodes sRGB, and multiplies alpha back in gamma space, because that is
how Android composites. Readback suits accent-sized canvases (tens of
thousands of pixels). A full-screen scene would want a surface instead, which
`GpuContext::create_window_surface` already supports.

## Build

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android   # rust-toolchain.toml lists them
cargo install cargo-ndk
ANDROID_NDK_HOME=... cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -P 26 -o jniLibs \
  build -p water-ui-android --profile android-release
```

Each ABI produces a stripped `libwater_ui_android.so` of about 5–6 MB.
`.cargo/config.toml` links the 64-bit ABIs at 16 KB page alignment, which
Google Play requires for Android 15 devices whatever the NDK's default. The
`android-release` profile uses link-time optimisation and keeps panics
unwinding. The adapter requests Vulkan only: GLES lacks the storage-buffer
limits the simulation needs. On a device without Vulkan, or where the device
refuses the default limits, `engineCreate` returns 0 or throws `InitFailed`,
and the host falls back.

The device is requested with `wgpu::Limits::default()`, which is the
desktop baseline and stricter than the shaders need. A driver that reports
less than that baseline is refused even when it could run the engine.
Requesting the computed minimum is a known follow-up.

## Host behaviour to mirror

The web package's player resolves `live → baked → static → none`. An Android
host should follow the same rules:

- Draw the baked still while the engine is cold, so the engine is never
  awaited.
- Prefer the still under reduced motion.
- Fall back to the baked strip or the still on `InstanceLimit` or a device
  error.
- Pick the detail tier from the backing long edge. The tiers are below 64 px,
  below 192 px, below 320 px, and 320 px and up. Cap the density at 2.
- Pass `1 - tail` as `paintWallFraction`. The default tail is 0.8 and the
  default accent duration is 1200 ms.

The baked assets in `packages/water-ui/assets` are WebP, which Android decodes
natively, and a host can bundle them unchanged.
