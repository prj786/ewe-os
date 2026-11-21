<script>
  // ewe-installer — one decision per screen (RFC-003). The rail on the left
  // is orientation, the single button row at the bottom is the only
  // navigation, and nothing installs until the Summary's explicit button.
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { choices, step, visibleSteps, addonsCatalog, langData } from "./lib/state.js";
  import { layoutStrings, suggestLayouts, enginesFor } from "./lib/lang.js";
  import { curatedCategories } from "./lib/curated.js";
  import Language from "./lib/steps/Language.svelte";
  import Keyboard from "./lib/steps/Keyboard.svelte";
  import Apps from "./lib/steps/Apps.svelte";
  import Network from "./lib/steps/Network.svelte";
  import TimePlace from "./lib/steps/TimePlace.svelte";
  import Disk from "./lib/steps/Disk.svelte";
  import User from "./lib/steps/User.svelte";
  import Cloud from "./lib/steps/Cloud.svelte";
  import Addons from "./lib/steps/Addons.svelte";
  import Summary from "./lib/steps/Summary.svelte";
  import Install from "./lib/steps/Install.svelte";

  const COMPONENTS = { language: Language, keyboard: Keyboard, network: Network, timeplace: TimePlace, disk: Disk,
                       user: User, cloud: Cloud, addons: Addons, apps: Apps, summary: Summary, install: Install };

  // The first-party plugin catalogue is read once, from the live payload —
  // an older ewe on the ISO has none, and then the Plugins step simply is
  // not there.
  // Outside Tauri (`npm run dev` in a browser) `?mock` serves the fixture
  // and `?step=<key>` opens a screen directly — the screenshot recipe.
  onMount(async () => {
    const q = new URLSearchParams(location.search);
    loadLanguages(q);
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

  // The Language and Keyboard screens' data, read once from the live system
  // (in a plain browser with ?mock: a small fixture). Each part on its own —
  // a missing one leaves its suggestions out, never the screen.
  async function loadLanguages(q) {
    const mock = q.has("mock") && !("__TAURI_INTERNALS__" in window);
    const get = async (cmd) => {
      if (mock) {
        const m = await import("./lib/mock-lang.js");
        return { languages: m.MOCK_LANGUAGES, keyboard_db: m.MOCK_KBDB, ime_catalog: m.MOCK_IME }[cmd];
      }
      try { const r = await invoke(cmd); return Array.isArray(r) ? r : []; } catch { return []; }
    };
    const [rows, kbdb, ime] = await Promise.all([get("languages"), get("keyboard_db"), get("ime_catalog")]);
    langData.set({ rows, kbdb, ime });
    if (mock && q.has("lang")) {
      // mock: ?lang=ka_GE.UTF-8 pre-picks a language the way the screen would
      const r = rows.find((x) => x.locale === q.get("lang"));
      if (r) {
        const e = enginesFor(r.lang, ime);
        $choices.langCode = r.lang; $choices.locale = r.locale; $choices.formats = r.locale;
        $choices.dict = r.dict && r.dict.name !== "en_US" ? r.dict : null;
        $choices.ime = e.length ? [e[0].id] : [];
        $choices.layouts = suggestLayouts(r, kbdb, $choices.ime.length > 0);
      }
    }
    if (mock && q.has("apps")) {
      const all = curatedCategories.flatMap((c) => c.apps);
      $choices.apps = q.get("apps").split(",").map((p) => all.find((a) => a.pkg === p)).filter(Boolean)
        .map((a) => ({ pkg: a.pkg, name: a.name, source: a.source }));
    }
  }

  // The chosen keyboards, live in this session from the moment they are
  // chosen: the try-it field tests them, and the password on "Your login" is
  // typed the way the new computer's login screen will read it.
  let appliedLayouts = "";
  $: {
    const [l, v] = layoutStrings($choices.layouts);
    const key = `${l}|${v}`;
    if (l && key !== appliedLayouts && "__TAURI_INTERNALS__" in window) {
      appliedLayouts = key;
      invoke("try_layouts", { layouts: l, variants: v }).catch(() => {});
    }
  }

  // per-step gate: can the user proceed?
  $: c = $choices;
  $: steps = $visibleSteps;
  $: current = steps[$step] || steps[0];
  $: canNext = {
    language: true,      // a language is always picked (English to start with)
    keyboard: c.layouts.length > 0,
    network: c.online,   // the install downloads everything — no network, no erase
    timeplace: c.timezone !== "" && (c.tzConfident || c.tzConfirmed),
    disk: c.disk !== null,
    user: c.username !== "" && c.password.length >= 4,
    cloud: !c.signingIn,  // signed in or skipped — both are fine; not while the browser step waits
    addons: true,        // nothing picked is a fine answer
    apps: true,          // the same
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
    <div class="px-4 pb-4 text-xs text-zinc-600">beta — every choice is shown again before anything touches a disk</div>
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
