<script lang="ts">
  import { PALETTE_IDS, PRESENTATIONS, type PaletteId } from 'water-ui';
  import { Moon, Sun } from 'lucide';
  import { ui } from '../ui.svelte';
  import LucideIcon from './LucideIcon.svelte';

  const field = 'rounded-md border border-ink/15 bg-paper px-2 py-1 text-sm dark:border-mist/20 dark:bg-night-raised';
</script>

<header class="sticky top-0 z-20 border-b border-ink/10 bg-paper/85 backdrop-blur dark:border-mist/10 dark:bg-night/85">
  <div class="mx-auto flex max-w-6xl flex-wrap items-center gap-x-6 gap-y-2 px-6 py-3">
    <a href="#top" class="font-serif text-lg font-semibold">water-ui</a>
    <nav class="hidden gap-4 text-sm text-ink-soft md:flex dark:text-mist-soft">
      <a href="#gallery" class="hover:text-ink dark:hover:text-mist">Gallery</a>
      <a href="#icons" class="hover:text-ink dark:hover:text-mist">Icons</a>
      <a href="#accents" class="hover:text-ink dark:hover:text-mist">Accents</a>
      <a href="#drops" class="hover:text-ink dark:hover:text-mist">Paint drops</a>
      <a href="#usage" class="hover:text-ink dark:hover:text-mist">Usage</a>
    </nav>
    <div class="ml-auto flex flex-wrap items-center gap-2 text-sm sm:gap-3">
      <label class="flex items-center gap-2">
        <span class="hidden sm:inline">Palette</span>
        <select
          aria-label="Palette"
          class={field}
          value={ui.palette ?? ''}
          onchange={(e) => (ui.palette = (e.currentTarget.value || undefined) as PaletteId | undefined)}
        >
          <option value="">Default</option>
          {#each PALETTE_IDS as palette (palette)}
            <option value={palette}>{palette}</option>
          {/each}
        </select>
      </label>
      <label class="flex items-center gap-2">
        <span class="hidden sm:inline">Motion</span>
        <select aria-label="Motion" class={field} value={ui.presentation} onchange={(e) => ui.setPresentation(e.currentTarget.value as typeof ui.presentation)}>
          {#each PRESENTATIONS as presentation (presentation)}
            <option value={presentation}>{presentation}</option>
          {/each}
        </select>
      </label>
      <button
        class="{field} flex items-center gap-1.5"
        onclick={() => ui.setSurface(ui.surface === 'dark' ? 'light' : 'dark')}
        aria-label="Toggle dark mode"
      >
        <LucideIcon icon={ui.surface === 'dark' ? Sun : Moon} class="size-4" />
        <span class="hidden sm:inline">{ui.surface === 'dark' ? 'Light' : 'Dark'}</span>
      </button>
    </div>
  </div>
</header>
