//! Android adapter for the watercolour engine. `host` is the platform-neutral
//! half (device, instances, bitmap pixel format) the host-side tests drive;
//! `jni_api` is the JNI surface the Kotlin binding in `kotlin/` calls, and
//! compiles everywhere so clippy and the tests cover it off-device too.

pub mod host;
mod jni_api;
