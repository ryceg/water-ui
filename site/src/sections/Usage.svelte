<script lang="ts">
  import Section from './Section.svelte';

  const install = `git clone https://github.com/ryceg/water-ui
cd water-ui
pnpm install
pnpm build:wasm   # needs Rust, the wasm32 target and wasm-bindgen-cli`;

  const dependency = `{
  "dependencies": {
    "water-ui": "file:../water-ui/packages/water-ui"
  }
}`;

  const component = `<script lang="ts">
  import { Artwork, getEngineHost } from 'water-ui';

  getEngineHost().warmWhenIdle();
</${'script'}>

<Artwork artwork="crescent-moon" palette="moonlight" class="size-40" />`;

  const block = 'overflow-x-auto rounded-xl bg-ink/5 p-4 text-sm dark:bg-mist/5';
</script>

<Section
  id="usage"
  title="Usage"
  lede="The package ships Svelte and TypeScript source, so your Vite build compiles it. Build the wasm once in a checkout, then depend on it by path."
>
  <div class="grid gap-6 lg:grid-cols-3">
    <div class="min-w-0">
      <h3 class="mb-2 font-medium">1. Build</h3>
      <pre class={block}><code>{install}</code></pre>
    </div>
    <div class="min-w-0">
      <h3 class="mb-2 font-medium">2. Depend on it</h3>
      <pre class={block}><code>{dependency}</code></pre>
    </div>
    <div class="min-w-0">
      <h3 class="mb-2 font-medium">3. Paint</h3>
      <pre class={block}><code>{component}</code></pre>
    </div>
  </div>
  <p class="mt-6 text-sm text-ink-soft dark:text-mist-soft">
    Peer dependencies: <code>svelte</code> 5, <code>tailwindcss</code> 4 and <code>lucide</code>. See the
    <a class="underline" href="https://github.com/ryceg/water-ui/blob/main/docs/public-api.md">public API</a> and the
    <a class="underline" href="https://github.com/ryceg/water-ui/blob/main/docs/architecture.md">architecture</a> notes.
  </p>
</Section>
