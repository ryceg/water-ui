# water-ui

**Showcase: https://ryceg.github.io/water-ui/**

Watercolour artwork for web UIs. A WebGPU simulation, written in Rust and
compiled to wasm, paints each artwork in front of the viewer. Where WebGPU is
missing, capped or refused, the same artwork plays from a baked frame strip or
shows as a still. The public surface is plain TypeScript with thin Svelte 5
components.

| Path | Contents |
|---|---|
| `crates/water-ui-core/` | Domain model, CPU reference simulation, Kubelka-Munk optics, playback. `std` only, no dependencies. |
| `crates/water-ui-infra/` | wgpu/WGSL simulation and rendering, scene documents, PNG export, the artwork catalogue and Lucide icon authoring. |
| `crates/water-ui-wasm/` | wasm-bindgen web adapter. |
| `crates/water-ui-android/` | JNI Android adapter: Vulkan device, instances rendered into bitmaps, and its Kotlin binding. See [docs/android.md](docs/android.md). |
| `packages/water-ui/` | The `water-ui` npm package: TypeScript API, Svelte components, baked assets, bench harness. |
| `site/` | The showcase site: Vite + Svelte 5, deployed to GitHub Pages by `.github/workflows/pages.yml`. `pnpm site` runs it locally after `pnpm build:wasm`. |
| `docs/` | Architecture, scene format, pigment model, public API, verification. Start at [docs/README.md](docs/README.md). |

## Build

Needs Rust (the toolchain is pinned in `rust-toolchain.toml`), the
`wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.128 (the version pinned
in `Cargo.toml`), Node 24 and pnpm. `wasm-opt` (binaryen) on `PATH` is optional
and shrinks the module.

```bash
cargo install wasm-bindgen-cli --version 0.2.128
pnpm install
pnpm build:wasm   # cargo + wasm-bindgen into packages/water-ui/src/wasm (gitignored)
pnpm check
pnpm test
cargo test --workspace
```

## Using it from another project

The package ships TypeScript and Svelte source (`exports` point at
`src/index.ts`), so the consumer's Vite/SvelteKit build compiles it. The wasm
bindings are not committed, so run `pnpm build:wasm` in this checkout first,
then depend on the package by path:

```json
{ "dependencies": { "water-ui": "file:../water-ui/packages/water-ui" } }
```

Peer dependencies: `svelte` >= 5, `tailwindcss` >= 4 and `lucide` >= 0.400.
See [docs/public-api.md](docs/public-api.md) for the API and a host-app example.

## Licence

No licence is granted. All rights reserved. Third-party material is listed in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
