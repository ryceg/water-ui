<script lang="ts">
  import { Artwork } from 'water-ui';
  import { BookOpen, Cpu, Database, Fingerprint, Flag, Megaphone, Rocket, Sprout, type IconNode } from 'lucide';
  import { ui } from '../ui.svelte';
  import LucideIcon from './LucideIcon.svelte';
  import Section from './Section.svelte';

  const ICONS: { name: string; icon: IconNode }[] = [
    { name: 'database', icon: Database },
    { name: 'cpu', icon: Cpu },
    { name: 'fingerprint', icon: Fingerprint },
    { name: 'sprout', icon: Sprout },
    { name: 'flag', icon: Flag },
    { name: 'megaphone', icon: Megaphone },
    { name: 'rocket', icon: Rocket },
    { name: 'book-open', icon: BookOpen },
  ];
</script>

<Section
  id="icons"
  title="Any Lucide icon"
  lede="Hand any Lucide element list to Artwork and it is authored into a watercolour scene: closed shapes become washed bodies, open paths become brush marks. With painting turned off, the plain icon stays."
>
  <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
    {#each ICONS as { name, icon } (name)}
      <div class="flex items-center gap-4 rounded-xl border border-ink/10 p-4 dark:border-mist/10">
        <LucideIcon {icon} class="size-6 shrink-0 text-ink-soft dark:text-mist-soft" />
        <span class="text-ink-soft dark:text-mist-soft">→</span>
        <div class="size-20">
          {#key ui.surface}
            <Artwork icon={{ icon, name }} palette={ui.palette} surface={ui.surface} class="h-full w-full" />
          {/key}
        </div>
      </div>
    {/each}
  </div>
  <pre class="mt-6 overflow-x-auto rounded-xl bg-ink/5 p-4 text-sm dark:bg-mist/5"><code>{`import { Database } from 'lucide';

<Artwork icon={{ icon: Database, name: 'database' }} class="size-20" />`}</code></pre>
</Section>
