<script lang="ts">
  import { AvatarWash, ConfirmationBackground, PaintedUnderline, ReportHeaderWash, SelectionEdge } from 'water-ui';
  import { ui } from '../ui.svelte';
  import Section from './Section.svelte';

  const TABS = ['Overview', 'Activity', 'Reports', 'Settings'];
  const NAV = ['Inbox', 'Drafts', 'Archive', 'Shared with me'];

  let tab = $state(0);
  let nav = $state(0);
  let names = $state(['Ada Lovelace', 'Hokusai', 'Winslow Homer', 'Turner', 'Georgia', 'Mei']);
  let draft = $state('');
  let saves = $state(0);
  let washName = $state('quarterly report');

  function addName(event: SubmitEvent) {
    event.preventDefault();
    const name = draft.trim();
    if (name) names = [...names, name];
    draft = '';
  }

  const card = 'min-w-0 rounded-xl border border-ink/10 bg-white/50 dark:border-mist/10 dark:bg-night-raised/60';
  const caption = 'mt-3 text-sm text-ink-soft dark:text-mist-soft';
  const input = 'rounded-md border border-ink/15 bg-paper px-2 py-1 text-sm dark:border-mist/20 dark:bg-night';
</script>

<Section
  id="accents"
  title="Accents"
  lede="Small marks for ordinary interface states. Each paints once when it becomes relevant, then lets go of the GPU."
>
  <div class="grid gap-6 md:grid-cols-2">
    <div class="{card} p-5">
      <div class="flex gap-1 overflow-x-auto border-b border-ink/10 dark:border-mist/10" role="tablist">
        {#each TABS as label, i (label)}
          <button class="relative shrink-0 px-3 py-2.5 text-sm sm:px-4" role="tab" aria-selected={tab === i} onclick={() => (tab = i)}>
            {label}
            <PaintedUnderline active={tab === i} seed={i + 1} palette={ui.palette} surface={ui.surface} />
          </button>
        {/each}
      </div>
      <p class={caption}><code>PaintedUnderline</code> brushes under the active tab.</p>
    </div>

    <div class="{card} p-5">
      <nav class="flex flex-col">
        {#each NAV as label, i (label)}
          <button class="relative py-2 pl-5 text-left text-sm" onclick={() => (nav = i)}>
            {label}
            <SelectionEdge active={nav === i} seed={i + 1} palette={ui.palette} surface={ui.surface} />
          </button>
        {/each}
      </nav>
      <p class={caption}><code>SelectionEdge</code> marks the selected item.</p>
    </div>

    <div class="{card} p-5">
      <div class="flex flex-wrap gap-3">
        {#each names as name, i (`${name}-${i}`)}
          <div class="flex items-center gap-2 text-sm">
            {#key ui.surface}
              <AvatarWash {name} size={36} palette={ui.palette} surface={ui.surface} />
            {/key}
            {name}
          </div>
        {/each}
      </div>
      <form class="mt-4 flex gap-2" onsubmit={addName}>
        <input class="{input} flex-1" placeholder="Add a name" bind:value={draft} />
        <button class="{input} px-3">Add</button>
      </form>
      <p class={caption}><code>AvatarWash</code> seeds a wash from each name, so a name always gets the same one.</p>
    </div>

    <div class="{card} relative overflow-hidden p-5">
      {#if saves > 0}
        {#key `${saves}-${ui.surface}`}
          <ConfirmationBackground palette={ui.palette} surface={ui.surface} />
        {/key}
      {/if}
      <div class="relative">
        <p class="font-medium">{saves > 0 ? 'Saved. Your changes are live.' : 'Unsaved changes'}</p>
        <button
          class="mt-4 rounded-full bg-ink px-4 py-2 text-sm font-medium text-paper dark:bg-mist dark:text-night"
          onclick={() => saves++}
        >
          Save
        </button>
        <p class={caption}><code>ConfirmationBackground</code> washes in behind a success state.</p>
      </div>
    </div>

    <div class="{card} relative overflow-hidden md:col-span-2">
      <ReportHeaderWash name={washName} position="absolute" class="inset-0 opacity-60" />
      <div class="relative flex flex-wrap items-center gap-4 p-5">
        <h3 class="font-serif text-2xl capitalize">{washName || 'Untitled'}</h3>
        <input class="{input} ml-auto" bind:value={washName} aria-label="Header name" />
      </div>
      <p class="{caption} relative px-5 pb-5">
        <code>ReportHeaderWash</code> generates a wash for its box, seeded from a name. Type to repaint.
      </p>
    </div>
  </div>
</Section>
