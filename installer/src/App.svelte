<script>
  // ewe-installer — one decision per screen (RFC-003). The rail on the left
  // is orientation, the single button row at the bottom is the only
  // navigation, and nothing installs until the Summary's explicit button.
  import { choices, step, STEPS } from "./lib/state.js";
  import Welcome from "./lib/steps/Welcome.svelte";
  import TimePlace from "./lib/steps/TimePlace.svelte";
  import Disk from "./lib/steps/Disk.svelte";
  import User from "./lib/steps/User.svelte";
  import Summary from "./lib/steps/Summary.svelte";
  import Install from "./lib/steps/Install.svelte";

  const COMPONENTS = [Welcome, TimePlace, Disk, User, Summary, Install];

  // per-step gate: can the user proceed?
  $: c = $choices;
  $: canNext = [
    true,
    c.timezone !== "",
    c.disk !== null,
    c.username !== "" && c.password.length >= 4,
    true,   // Summary's own button advances
    false,  // Install is terminal
  ][$step];

  $: installing = $step === STEPS.length - 1;
</script>

<div class="flex h-screen select-none bg-zinc-950 text-zinc-100">
  <!-- rail -->
  <aside class="flex w-56 flex-col gap-1 border-r border-zinc-800 p-5">
    <div class="mb-4 flex items-center gap-2">
      <img src="/usr/share/ewe/system/branding/ewe-logo-dark.png" alt="" class="h-8 w-8 opacity-90"
           onerror={(e) => (e.target.style.display = "none")} />
      <span class="text-lg font-bold tracking-tight">Install ewe</span>
    </div>
    {#each STEPS as s, i}
      <div class="flex items-center gap-2 rounded-lg px-3 py-1.5 text-sm
                  {i === $step ? 'bg-zinc-800 text-white' : i < $step ? 'text-zinc-400' : 'text-zinc-600'}">
        <span class="w-4 text-center text-xs {i < $step ? 'text-[var(--accent,#0a84ff)]' : ''}">{i < $step ? "\u2713" : i === $step ? "\u203a" : "\u00b7"}</span>
        {s.label}
      </div>
    {/each}
    <div class="mt-auto text-xs text-zinc-600">alpha — every choice is shown again before anything touches a disk</div>
  </aside>

  <!-- step -->
  <main class="flex min-w-0 flex-1 flex-col">
    <div class="min-h-0 flex-1 overflow-y-auto p-8">
      <svelte:component this={COMPONENTS[$step]} />
    </div>
    {#if !installing}
      <div class="flex items-center justify-between border-t border-zinc-800 px-8 py-4">
        <button class="rounded-lg px-4 py-2 text-sm text-zinc-400 hover:bg-zinc-900 disabled:opacity-30"
                disabled={$step === 0} onclick={() => step.update((n) => n - 1)}>Back</button>
        {#if $step < STEPS.length - 2}
          <button class="rounded-lg bg-[var(--accent,#0a84ff)] px-5 py-2 text-sm font-medium text-white disabled:opacity-30"
                  disabled={!canNext} onclick={() => step.update((n) => n + 1)}>Next</button>
        {/if}
      </div>
    {/if}
  </main>
</div>
