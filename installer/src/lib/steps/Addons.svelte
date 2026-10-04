<script>
  // Add-ons — every first-party extra the ewe payload carries, NONE of them
  // pre-checked (ewe 0.25: add-ons are opt-in; the same list and words as
  // the desktop's Welcome screen). The catalogue comes from the live ISO's
  // own payload (App.svelte → `addons`); picking here only records ids —
  // they are installed for the new account at the end of the run, each one
  // best-effort, so a failing add-on can never fail the OS install.
  import { choices, addonsCatalog } from "../state.js";
  import { glyphFor, groupAddons, GLYPH_CHECK, DOCK_ID, DOCK_HINT } from "../addons.js";

  $: groups = groupAddons($addonsCatalog || []);
  $: picked = new Set($choices.addons);

  function toggle(id) {
    const s = new Set($choices.addons);
    if (s.has(id)) s.delete(id); else s.add(id);
    // keep the catalogue's order, so the Summary reads the same way as this list
    $choices.addons = ($addonsCatalog || []).map((a) => a.id).filter((x) => s.has(x));
  }
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Add-ons</h1>
<p class="mb-6 max-w-xl text-sm text-zinc-400">
  A few extras ship with ewe — the dock, music, Places, your phone, mail, Cast to TV and more.
  None is installed until you pick it. Choose what you want now; the rest is one click away in Komble → Add-ons.
</p>

<div class="max-w-xl">
  {#each groups as g (g.category)}
    <div class="mb-2 mt-4 text-xs font-semibold uppercase tracking-wider text-zinc-500 first:mt-0">{g.category}</div>
    {#each g.items as a (a.id)}
      <button class="mb-2 flex w-full items-center gap-3 rounded-xl border px-4 py-3 text-left transition-colors
                     {picked.has(a.id) ? 'border-[var(--accent)] bg-zinc-800/60' : 'border-zinc-700/60 hover:bg-zinc-800/60'}"
              role="checkbox" aria-checked={picked.has(a.id)} onclick={() => toggle(a.id)}>
        <span class="icon flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-zinc-800 text-[20px] text-zinc-200">{glyphFor(a.icon)}</span>
        <span class="min-w-0 flex-1">
          <span class="block text-sm font-medium">{a.name || a.id}</span>
          <span class="block truncate text-xs text-zinc-500">{a.description || ""}</span>
          {#if a.id === DOCK_ID}
            <span class="block text-xs" style="color: var(--accent)">{DOCK_HINT}</span>
          {/if}
        </span>
        <!-- the checkbox: a bordered box that fills with the accent + a Lucide tick -->
        <span class="icon flex h-5 w-5 shrink-0 items-center justify-center rounded-md border text-[14px] text-white transition-colors"
              style={picked.has(a.id) ? "background: var(--accent); border-color: var(--accent)" : "border-color: var(--color-zinc-500)"}>
          {picked.has(a.id) ? String.fromCodePoint(GLYPH_CHECK) : ""}
        </span>
      </button>
    {/each}
  {/each}
</div>

<p class="mt-4 max-w-xl text-xs text-zinc-500">
  {$choices.addons.length === 0 ? "Nothing selected." : `${$choices.addons.length} selected.`}
  You can add or remove these any time in Komble → Add-ons.
</p>
