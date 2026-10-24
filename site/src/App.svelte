<script lang="ts">
  import Accents from './sections/Accents.svelte';
  import Drops from './sections/Drops.svelte';
  import Gallery from './sections/Gallery.svelte';
  import Hero from './sections/Hero.svelte';
  import Icons from './sections/Icons.svelte';
  import Toolbar from './sections/Toolbar.svelte';
  import Usage from './sections/Usage.svelte';
</script>

<Toolbar />
<main>
  <Hero />
  <Gallery />
  <Icons />
  <Accents />
  <Drops />
  <Usage />
</main>
<footer class="mx-auto max-w-6xl px-6 py-12 text-sm text-ink-soft dark:text-mist-soft">
  water-ui by <a class="underline" href="https://github.com/ryceg">ryceg</a>. The simulation is an independent implementation of
  Curtis, Banks and Beier, <em>Computer-Generated Watercolor</em> (SIGGRAPH 1997). Icons from
  <a class="underline" href="https://lucide.dev">Lucide</a>.
</footer>
