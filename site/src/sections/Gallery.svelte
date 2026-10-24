<script lang="ts">
  import { ARTWORK_IDS, Artwork, type ArtworkId } from 'water-ui';
  import { ui } from '../ui.svelte';
  import Section from './Section.svelte';

  // The accents get their own section; the rest are standalone pieces.
  const ACCENTS: ArtworkId[] = ['tab-underline', 'selection-edge', 'confirmation-background', 'avatar-wash', 'wash'];
  const SCENES: ArtworkId[] = ['moonlit-shoreline', 'distant-mountains', 'connected-shores', 'hub-dawn-ridges', 'overlapping-shapes'];
  const pieces = ARTWORK_IDS.filter((id) => !ACCENTS.includes(id) && !SCENES.includes(id));

  let seeds: Record<string, number> = $state({});
  const label = (id: string) => id.replace(/-/g, ' ');
</script>

<Section id="gallery" title="Gallery" lede="The built-in catalogue. Click any piece to repaint it with a new seed: the brushwork changes, the subject does not.">
  <h3 class="mb-4 font-serif text-xl">Scenes</h3>
  <div class="grid gap-6 sm:grid-cols-2">
    {#each SCENES as id (id)}
      {@render tile(id, 'aspect-[3/2]')}
    {/each}
  </div>
  <h3 class="mt-12 mb-4 font-serif text-xl">Objects</h3>
  <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-5">
    {#each pieces as id (id)}
      {@render tile(id, 'aspect-square')}
    {/each}
  </div>
</Section>

{#snippet tile(id: ArtworkId, aspect: string)}
  <button
    class="group flex flex-col rounded-xl border border-ink/10 bg-white/50 p-3 text-left transition hover:border-ink/25 hover:bg-white dark:border-mist/10 dark:bg-night-raised/60 dark:hover:border-mist/25 dark:hover:bg-night-raised"
    onclick={() => (seeds[id] = (seeds[id] ?? 1) + 1)}
    title="Repaint"
  >
    <div class={aspect}>
      {#key `${seeds[id] ?? 1}-${ui.surface}`}
        <Artwork artwork={id} seed={seeds[id]} palette={ui.palette} surface={ui.surface} class="h-full w-full" />
      {/key}
    </div>
    <span class="mt-2 text-sm text-ink-soft capitalize group-hover:text-ink dark:text-mist-soft dark:group-hover:text-mist">{label(id)}</span>
  </button>
{/snippet}
