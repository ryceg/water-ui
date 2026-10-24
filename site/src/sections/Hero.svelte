<script lang="ts">
  import { ArtworkHero, detectCapabilities, type Capabilities } from 'water-ui';
  import { ui } from '../ui.svelte';

  let caps: Capabilities | undefined = $state();
  detectCapabilities().then((value) => (caps = value));

  let replay = $state(0);

  const status = $derived(
    !caps
      ? 'Checking for WebGPU...'
      : caps.webgpu && caps.adapter
        ? 'WebGPU is available: these artworks are being simulated and painted live in your browser.'
        : `WebGPU is unavailable here${caps.reason ? ` (${caps.reason})` : ''}, so you are seeing baked frame strips of the same reveals.`,
  );
</script>

<div id="top" class="relative mx-auto max-w-6xl px-6 pt-16 pb-8">
  {#key `${replay}-${ui.surface}-${ui.palette}`}
    <ArtworkHero artwork="moonlit-shoreline" size={560} palette={ui.palette} surface={ui.surface} />
  {/key}
  <p class="text-sm font-medium tracking-widest text-ink-soft uppercase dark:text-mist-soft">Rust · WebGPU · Svelte 5</p>
  <h1 class="mt-3 font-serif text-5xl leading-tight font-semibold tracking-tight md:text-6xl">
    Watercolour,<br />painted as the page loads.
  </h1>
  <p class="mt-6 max-w-xl text-lg text-ink-soft dark:text-mist-soft">
    water-ui is a fluid simulation of pigment, water and paper, after Curtis et al.'s <em>Computer-Generated Watercolor</em>,
    compiled from Rust to WebAssembly and run on the GPU. Each artwork is a timeline of brush strokes that floods, blooms and
    dries on the screen in front of you.
  </p>
  <p class="mt-4 max-w-xl text-sm text-ink-soft dark:text-mist-soft">{status}</p>
  <div class="mt-8 flex flex-wrap gap-3">
    <a
      href="#gallery"
      class="rounded-full bg-ink px-5 py-2.5 text-sm font-medium text-paper hover:bg-ink/85 dark:bg-mist dark:text-night dark:hover:bg-mist/85"
    >
      Browse the gallery
    </a>
    <button
      class="rounded-full border border-ink/20 px-5 py-2.5 text-sm font-medium hover:bg-ink/5 dark:border-mist/25 dark:hover:bg-mist/10"
      onclick={() => replay++}
    >
      Paint it again
    </button>
    <a
      href="https://github.com/ryceg/water-ui"
      class="rounded-full border border-ink/20 px-5 py-2.5 text-sm font-medium hover:bg-ink/5 dark:border-mist/25 dark:hover:bg-mist/10"
    >
      Source on GitHub
    </a>
  </div>
  <div class="clear-both"></div>
</div>
