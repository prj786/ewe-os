<script>
  // ewe-installer — one decision per screen (RFC-003). The rail on the left
  // is orientation, the single button row at the bottom is the only
  // navigation, and nothing installs until the Summary's explicit button.
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { choices, step, visibleSteps, addonsCatalog } from "./lib/state.js";
  import Welcome from "./lib/steps/Welcome.svelte";
  import Network from "./lib/steps/Network.svelte";
  import TimePlace from "./lib/steps/TimePlace.svelte";
  import Disk from "./lib/steps/Disk.svelte";
  import User from "./lib/steps/User.svelte";
  import Addons from "./lib/steps/Addons.svelte";
  import Summary from "./lib/steps/Summary.svelte";
  import Install from "./lib/steps/Install.svelte";

  const COMPONENTS = { welcome: Welcome, network: Network, timeplace: TimePlace, disk: Disk,
                       user: User, addons: Addons, summary: Summary, install: Install };

  // The add-on catalogue is read once, from the live payload — an older ewe
  // on the ISO has none, and then the Add-ons step simply is not there.
  // Outside Tauri (`npm run dev` in a browser) `?mock` serves the fixture
  // and `?step=<key>` opens a screen directly — the screenshot recipe.
  onMount(async () => {
    const q = new URLSearchParams(location.search);
    try {
      const r = await invoke("addons");
      addonsCatalog.set(Array.isArray(r) ? r : []);
    } catch {
      if (q.has("mock") && !("__TAURI_INTERNALS__" in window)) {
        const { MOCK_ADDONS } = await import("./lib/mock.js");
        addonsCatalog.set(MOCK_ADDONS);
      } else {
        addonsCatalog.set([]);
      }
    }
    if (q.has("pick")) $choices.addons = q.get("pick").split(",").filter(Boolean);   // mock: pre-picked ids
    if (q.has("step")) step.set(Math.max(0, $visibleSteps.findIndex((s) => s.key === q.get("step"))));
  });

  // per-step gate: can the user proceed?
  $: c = $choices;
  $: steps = $visibleSteps;
  $: current = steps[$step] || steps[0];
  $: canNext = {
    welcome: true,
    network: c.online,   // the install downloads everything — no network, no erase
    timeplace: c.timezone !== "" && (c.tzConfident || c.tzConfirmed),
    disk: c.disk !== null,
    user: c.username !== "" && c.password.length >= 4,
    addons: true,        // nothing picked is a fine answer
    summary: true,       // Summary's own button advances
    install: false,      // Install is terminal
  }[current.key];

  $: installing = current.key === "install";
</script>

<div class="flex h-full">
  <!-- rail — same shape as ewe-settings' sidebar: Lucide glyphs, accent on
       the active row, a check where a decision is already made -->
  <aside class="flex w-56 shrink-0 flex-col border-r border-zinc-800/60 bg-black/20">
    <div class="flex items-center gap-2 px-4 pb-2 pt-5">
      <img src="/usr/share/ewe/system/branding/ewe-logo-dark.png" alt="" class="h-7 w-7 opacity-90"
           onerror={(e) => (e.target.style.display = "none")} />
      <span class="text-sm font-semibold">Install ewe</span>
    </div>
    <nav class="flex-1 space-y-0.5 px-2 py-2">
      {#each steps as s, i (s.key)}
        <div class="flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-left text-sm transition-colors
                    {i === $step ? 'text-white' : i < $step ? 'text-zinc-400' : 'text-zinc-600'}"
             style={i === $step ? "background: color-mix(in srgb, var(--accent) 22%, transparent)" : ""}>
          <span class="icon w-4 shrink-0 text-center text-[16px]"
                style={i < $step ? "color: var(--accent)" : ""}>{String.fromCodePoint(i < $step ? 0xE06C : s.icon)}</span>
          <span class="min-w-0 flex-1 truncate">{s.label}</span>
        </div>
      {/each}
    </nav>
    <div class="px-4 pb-4 text-xs text-zinc-600">alpha — every choice is shown again before anything touches a disk</div>
  </aside>

  <!-- step -->
  <main class="flex min-w-0 flex-1 flex-col">
    <div class="min-h-0 flex-1 overflow-y-auto p-8">
      <svelte:component this={COMPONENTS[current.key]} />
    </div>
    {#if !installing}
      <div class="flex items-center justify-between border-t border-zinc-800/60 px-8 py-4">
        <button class="btn-ghost" disabled={$step === 0} onclick={() => step.update((n) => n - 1)}>Back</button>
        {#if current.key !== "summary"}
          <button class="btn-primary px-5" disabled={!canNext} onclick={() => step.update((n) => n + 1)}>Next</button>
        {/if}
      </div>
    {/if}
  </main>
</div>
