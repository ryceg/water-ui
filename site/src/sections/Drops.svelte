<script lang="ts">
  import { DropGroup, DropSurface, type DropFonts } from 'water-ui';
  import { Brush, Droplets, Layers, Sparkles, Sun, Waves } from 'lucide';
  import { ui } from '../ui.svelte';
  import LucideIcon from './LucideIcon.svelte';
  import Section from './Section.svelte';

  const FEATURES = [
    { title: 'Wet into wet', copy: 'Water carries pigment across the sheet and blooms where it meets a drying edge.', icon: Droplets },
    { title: 'Kubelka-Munk optics', copy: 'Layers of pigment mix subtractively, the way glazes do on paper.', icon: Layers },
    { title: 'Paper grain', copy: 'A height field gives every wash its granulation and hard dried edges.', icon: Waves },
    { title: 'Brush strokes', copy: 'Strokes are fitted to the empty space around your content, never over it.', icon: Brush },
    { title: 'Dark surfaces', copy: 'A luminous compositing mode paints light onto dark grounds.', icon: Sun },
    { title: 'Graceful fallback', copy: 'No WebGPU? The same reveal plays from a baked strip, or shows a still.', icon: Sparkles },
  ];

  // Must match the classes on the text below exactly: the fitter lays the text out off the DOM.
  const FONTS: DropFonts = {
    title: { font: '600 15px Inter', lineHeight: 22 },
    copy: { font: '400 14px Inter', lineHeight: 20 },
  };
</script>

<Section
  id="drops"
  title="Paint drops"
  lede="Hover a card. DropSurface measures its content and paints one gesture in the space left over: a stroke, a splash of drops, a splotch or a border. These are live only, so they need WebGPU."
>
  <DropGroup name="features">
    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each FEATURES as feature (feature.title)}
        <DropSurface
          name={feature.title}
          fonts={FONTS}
          palette={ui.palette}
          class="overflow-hidden rounded-xl border border-ink/10 bg-white/50 dark:border-mist/10 dark:bg-night-raised/60"
          contentClass="flex items-start gap-3 p-5 pr-24"
        >
          <div data-drop-obstacle class="size-6 shrink-0 text-ink-soft dark:text-mist-soft">
            <LucideIcon icon={feature.icon} class="size-6" />
          </div>
          <div>
            <h3 data-drop-text="title" class="text-[15px] leading-[22px] font-semibold">{feature.title}</h3>
            <p data-drop-text="copy" class="mt-1 text-sm leading-5 text-ink-soft dark:text-mist-soft">{feature.copy}</p>
          </div>
        </DropSurface>
      {/each}
    </div>
  </DropGroup>
</Section>
