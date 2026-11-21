<script>
  // Language first (ewe-os 0.17) — the installer's first screen. A searchable
  // list of languages in their own words ("ქართული — Georgian"); picking one
  // fills in everything that follows from it: the region (the locale), the
  // keyboard (next screen), the formats (Time & place), a spelling
  // dictionary when Arch has one, and for Chinese, Japanese, Korean,
  // Vietnamese and the Indic languages an input method. All changeable.
  import { onMount, tick } from "svelte";
  import { choices, langData } from "../state.js";
  import {
    groupLanguages, searchLanguages, regionLabel, regionNative, suggestLayouts, enginesFor,
  } from "../lang.js";

  let query = "";
  let listEl;

  $: groups = groupLanguages($langData.rows || []);
  $: hits = searchLanguages(groups, query);
  $: current = groups.find((g) => g.lang === $choices.langCode) || null;
  $: row = current?.rows.find((r) => r.locale === $choices.locale) || current?.def || null;
  $: engines = row ? enginesFor(row.lang, $langData.ime) : [];
  $: typing = $choices.ime.length > 0;

  // picking a language (or one of its regions) re-suggests everything below
  function pickLocale(r) {
    const newLang = $choices.langCode !== r.lang;
    $choices.langCode = r.lang;
    $choices.locale = r.locale;
    $choices.formats = r.locale;
    $choices.dict = r.dict && r.dict.name !== "en_US" ? r.dict : null;
    if (newLang) {
      const e = enginesFor(r.lang, $langData.ime);
      $choices.ime = e.length ? [e[0].id] : [];
      $choices.layoutsTouched = false;
    }
    if (!$choices.layoutsTouched) $choices.layouts = suggestLayouts(r, $langData.kbdb, $choices.ime.length > 0);
  }

  function pick(h) {
    pickLocale(h.row || h.g.def);
  }

  function setTyping(on, id = engines[0]?.id) {
    $choices.ime = on && id ? [id] : [];
    if (row && !$choices.layoutsTouched) $choices.layouts = suggestLayouts(row, $langData.kbdb, on);
  }

  function onKey(e) {
    if (e.key === "Enter" && hits.length) { pick(hits[0]); query = ""; }
  }

  // the picked language in view (the list arriving, a return to this screen,
  // a pick made elsewhere) — only scrolled when it is out of sight
  async function reveal() {
    await tick();
    const el = listEl?.querySelector("[aria-selected='true']");
    if (!el || !listEl) return;
    const top = el.offsetTop, bottom = top + el.offsetHeight;
    if (top < listEl.scrollTop || bottom > listEl.scrollTop + listEl.clientHeight)
      listEl.scrollTop = top - listEl.clientHeight / 2 + el.offsetHeight / 2;
  }
  onMount(reveal);
  $: if ($langData.rows && $choices.langCode && !query) reveal();
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Welcome to ewe</h1>
<p class="mb-4 max-w-xl text-sm text-muted-foreground">
  Choose your language. The keyboard, formats and the rest follow from it; you can change each one.
</p>

<div class="relative mb-2 max-w-xl">
  <span class="icon pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[14px] text-muted-foreground">{String.fromCodePoint(0xE151)}</span>
  <input class="input pl-9" placeholder="Search: English, Deutsch, 日本語, ქართული…" bind:value={query} onkeydown={onKey}
         aria-label="Search languages" />
</div>

<div class="relative mb-3 max-h-44 max-w-xl overflow-y-auto rounded-xl border border-border bg-card" role="listbox"
     aria-label="Languages" bind:this={listEl}>
  {#if $langData.rows === null}
    <div class="px-3 py-2 text-sm text-muted-foreground">Reading the languages…</div>
  {:else if hits.length === 0}
    <div class="px-3 py-2 text-sm text-muted-foreground">No language matches “{query}”.</div>
  {/if}
  {#each hits as h (h.g.lang)}
    {@const sel = h.g.lang === $choices.langCode}
    <button class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors {sel ? '' : 'hover:bg-muted/60'}"
            style={sel ? "background: color-mix(in srgb, var(--accent) 22%, transparent)" : ""}
            role="option" aria-selected={sel} onclick={() => pick(h)}>
      <span class="font-medium" lang={h.g.lang}>{h.g.native}</span>
      {#if h.g.native !== h.g.english}<span class="truncate text-muted-foreground">— {h.g.english}</span>{/if}
      <span class="ml-auto flex shrink-0 items-center gap-2">
        {#if h.row}<span class="text-xs text-muted-foreground">{regionLabel(h.row)}</span>{/if}
        {#if sel}<span class="icon text-[14px]" style="color: var(--accent)">{String.fromCodePoint(0xE06C)}</span>{/if}
      </span>
    </button>
  {/each}
</div>

{#if current && row}
  <div class="card max-w-xl divide-y divide-border text-sm">
    <div class="flex items-center gap-3 px-4 py-2">
      <span class="w-28 shrink-0 text-muted-foreground">Region</span>
      {#if current.rows.length > 1}
        <select class="input flex-1" value={row.locale} aria-label="Region"
                onchange={(e) => pickLocale(current.rows.find((r) => r.locale === e.currentTarget.value))}>
          {#each current.rows as r (r.locale)}
            {@const nat = regionNative(r)}
            <option value={r.locale}>{regionLabel(r) || r.locale}{nat && nat !== regionLabel(r) ? ` (${nat})` : ""}</option>
          {/each}
        </select>
      {:else}
        <span class="flex-1">{regionLabel(row) || row.locale}</span>
      {/if}
    </div>

    <div class="flex items-start gap-3 px-4 py-2.5">
      <span class="w-28 shrink-0 text-muted-foreground">Spell checking</span>
      <div class="flex-1">
        {#if row.dict && row.dict.name !== "en_US"}
          <label class="flex items-center gap-2">
            <input type="checkbox" checked={$choices.dict !== null}
                   onchange={(e) => ($choices.dict = e.currentTarget.checked ? row.dict : null)} />
            <span>Check spelling in {current.native}, and in English</span>
          </label>
        {:else if row.lang === "en"}
          <span>English spell checking comes with ewe.</span>
        {:else if row.dict_aur}
          <span>Arch has no {current.english} dictionary of its own, so ewe checks English.</span>
          <span class="block text-xs text-muted-foreground">You can add the {current.english} one later in ewe Settings → Language.</span>
        {:else}
          <span>There is no {current.english} dictionary for Arch, so ewe checks English.</span>
        {/if}
      </div>
    </div>

    {#if engines.length}
      <div class="flex items-start gap-3 px-4 py-2.5">
        <span class="w-28 shrink-0 text-muted-foreground">Typing</span>
        <div class="flex-1">
          <label class="flex items-center gap-2">
            <input type="checkbox" checked={typing} onchange={(e) => setTyping(e.currentTarget.checked)} />
            <span>Type in <span lang={row.lang}>{engines[0].native || current.native}</span></span>
          </label>
          {#if typing && engines.length > 1}
            <select class="input mt-2" value={$choices.ime[0]} aria-label="How to type"
                    onchange={(e) => setTyping(true, e.currentTarget.value)}>
              {#each engines as en (en.id)}<option value={en.id}>{en.method}</option>{/each}
            </select>
          {/if}
          <span class="block text-xs text-muted-foreground">
            {#if typing}
              {engines.find((x) => x.id === $choices.ime[0])?.method || ""} — Ctrl+Space turns it on and off.
            {:else}
              Only what the keyboard shows. ewe Settings → Keyboard and mouse can turn it on later.
            {/if}
          </span>
        </div>
      </div>
    {/if}
  </div>
  <p class="mt-2 max-w-xl text-xs text-muted-foreground">
    Keyboard and formats come next, already filled in. ewe's own screens are in English for now; apps and the system use your language.
  </p>
{/if}
