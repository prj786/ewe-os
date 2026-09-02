<script>
  // ewe-installer — one decision per screen (RFC-003). The rail on the left
  // is orientation, the single button row at the bottom is the only
  // navigation, and nothing installs until the Summary's explicit button.
  import { choices, step, STEPS } from "./lib/state.js";
  import Welcome from "./lib/steps/Welcome.svelte";
  import Network from "./lib/steps/Network.svelte";
  import TimePlace from "./lib/steps/TimePlace.svelte";
  import Disk from "./lib/steps/Disk.svelte";
  import User from "./lib/steps/User.svelte";
  import Summary from "./lib/steps/Summary.svelte";
  import Install from "./lib/steps/Install.svelte";

  const COMPONENTS = [Welcome, Network, TimePlace, Disk, User, Summary, Install];

  // per-step gate: can the user proceed?
  $: c = $choices;
  $: canNext = [
    true,
    c.online,   // the install downloads everything — no network, no erase
    c.timezone !== "" && (c.tzConfident || c.tzConfirmed),
    c.disk !== null,
    c.username !== "" && c.password.length >= 4,
    true,   // Summary's own button advances
    false,  // Install is terminal
  ][$step];

  $: installing = $step === STEPS.length - 1;
</script>

<div class="flex h-full">
  <!-- rail — same shape as ewe-settings' sidebar: Phosphor glyphs, accent on
       the active row, a check where a decision is already made -->
  <aside class="flex w-56 shrink-0 flex-col border-r border-zinc-800/60 bg-black/20">
    <div class="flex items-center gap-2 px-4 pb-2 pt-5">
      <img src="/usr/share/ewe/system/branding/ewe-logo-dark.png" alt="" class="h-7 w-7 opacity-90"
           onerror={(e) => (e.target.style.display = "none")} />
      <span class="text-sm font-semibold">Install ewe</span>
    </div>
    <nav class="flex-1 space-y-0.5 px-2 py-2">
      {#each STEPS as s, i}
        <div class="flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-left text-sm transition-colors
                    {i === $step ? 'text-white' : i < $step ? 'text-zinc-400' : 'text-zinc-600'}"
             style={i === $step ? "background: color-mix(in srgb, var(--accent) 22%, transparent)" : ""}>
          <span class="ph-i w-4 shrink-0 text-center text-[16px]"
                style={i < $step ? "color: var(--accent)" : ""}>{String.fromCodePoint(i < $step ? 0xe182 : s.icon)}</span>
          <span class="min-w-0 flex-1 truncate">{s.label}</span>
        </div>
      {/each}
    </nav>
    <div class="px-4 pb-4 text-xs text-zinc-600">alpha — every choice is shown again before anything touches a disk</div>
  </aside>

  <!-- step -->
  <main class="flex min-w-0 flex-1 flex-col">
    <div class="min-h-0 flex-1 overflow-y-auto p-8">
      <svelte:component this={COMPONENTS[$step]} />
    </div>
    {#if !installing}
      <div class="flex items-center justify-between border-t border-zinc-800/60 px-8 py-4">
        <button class="btn-ghost" disabled={$step === 0} onclick={() => step.update((n) => n - 1)}>Back</button>
        {#if $step < STEPS.length - 2}
          <button class="btn-primary px-5" disabled={!canNext} onclick={() => step.update((n) => n + 1)}>Next</button>
        {/if}
      </div>
    {/if}
  </main>
</div>
