<script lang="ts">
  import type { Snippet } from 'svelte';

  let { id, title, lede, children }: { id: string; title: string; lede: string; children: Snippet } = $props();
</script>

<section {id} class="mx-auto max-w-6xl scroll-mt-20 px-6 py-16">
  <h2 class="font-serif text-3xl font-semibold tracking-tight">{title}</h2>
  <p class="mt-2 max-w-2xl text-ink-soft dark:text-mist-soft">{lede}</p>
  <div class="mt-8">{@render children()}</div>
</section>
