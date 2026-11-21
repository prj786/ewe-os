<script>
  // The only screen with an Install button. Every choice in plain words;
  // the destructive one in red.
  import { choices, step, addonsCatalog, langData } from "../state.js";
  import { groupLanguages, regionLabel, localeLabel, enginesFor } from "../lang.js";
  $: c = $choices;
  $: rows = $langData.rows || [];
  $: lang = groupLanguages(rows.filter((r) => r.lang === c.langCode))[0] || null;
  $: locRow = rows.find((r) => r.locale === c.locale) || null;
  $: fmtRow = rows.find((r) => r.locale === c.formats) || null;
  $: engine = (($langData.ime || []).find((e) => e.id === c.ime[0])) || null;
  $: hasEngines = enginesFor(c.langCode, $langData.ime).length > 0;
  $: repoApps = c.apps.filter((a) => a.source === "repo").map((a) => a.name);
  $: laterApps = c.apps.filter((a) => a.source !== "repo").map((a) => a.name);
  // the picked plugins by name, in the catalogue's order (ids only if the
  // catalogue vanished — it cannot, but the Summary must never show nothing)
  $: addonNames = c.addons.map((id) => ($addonsCatalog || []).find((a) => a.id === id)?.name || id);
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Summary</h1>
<p class="mb-6 text-sm text-zinc-400">Read it once — this is the moment before anything is touched.</p>

<div class="max-w-xl space-y-2 text-sm">
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Disk</span><span class="text-red-400">{c.disk?.path} ({c.disk?.size}) — will be completely erased</span></div>
  <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
    <span class="shrink-0 text-zinc-500">Language</span>
    <span class="text-right">{#if lang}<span lang={lang.lang}>{lang.native}</span>{lang.native !== lang.english ? ` — ${lang.english}` : ""}{locRow && regionLabel(locRow) ? ` (${regionLabel(locRow)})` : ""}{:else}{c.locale}{/if}</span>
  </div>
  <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
    <span class="shrink-0 text-zinc-500">Keyboard</span>
    <span class="text-right">{c.layouts.map((k) => k.n).join(", ")}{c.layouts.length > 1 ? " — Super+Space switches" : ""}</span>
  </div>
  <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
    <span class="shrink-0 text-zinc-500">Time & place</span>
    <span class="text-right">{c.timezone} · formats: {fmtRow ? localeLabel(fmtRow) : c.formats}</span>
  </div>
  <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
    <span class="shrink-0 text-zinc-500">Spell checking</span>
    <span class="text-right">{c.dict && locRow ? `${localeLabel(rows.find((r) => r.locale.startsWith(c.dict.name + ".")) || locRow)}, and English` : "English"}</span>
  </div>
  {#if hasEngines}
    <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
      <span class="shrink-0 text-zinc-500">Typing</span>
      <span class="text-right">{engine ? `${engine.native || engine.language} — ${engine.method}, Ctrl+Space turns it on` : "only what the keyboard shows"}</span>
    </div>
  {/if}
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Account</span><span>{c.realName} ({c.username}) on “{c.hostname}”</span></div>
  <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
    <span class="shrink-0 text-zinc-500">ewe account</span>
    <span class="text-right">
      {#if c.account}
        {c.account.name || c.account.login} on {String(c.account.server).replace(/^https?:\/\//, "")}{c.account.backup?.exists && c.account.restore ? ` — brings back everything from “${c.account.backup.machine || "your other computer"}”` : " — signed in"}
      {:else}
        not signed in — sign in from the desktop any time
      {/if}
    </span>
  </div>
  {#if $addonsCatalog !== null && $addonsCatalog.length > 0}
    <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
      <span class="shrink-0 text-zinc-500">Plugins</span>
      <span class="text-right">{addonNames.length ? addonNames.join(", ") : "none — add them later in Komble → Plugins"}</span>
    </div>
  {/if}
  <div class="flex justify-between gap-6 border-b border-zinc-800 pb-2">
    <span class="shrink-0 text-zinc-500">Apps</span>
    <span class="text-right">
      {#if c.apps.length === 0}none — Komble has them any time{:else}
        {repoApps.join(", ")}{repoApps.length && laterApps.length ? "; " : ""}{laterApps.length ? `${laterApps.join(", ")} waiting in Komble after sign-in` : ""}
      {/if}
    </span>
  </div>
</div>

<div class="card mt-6 max-w-xl p-4">
  <div class="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-500">Decided by ewe — nothing to configure</div>
  <div class="grid grid-cols-2 gap-x-6 gap-y-1 text-xs text-zinc-400">
    <span>Filesystem</span><span class="text-zinc-300">btrfs · compressed · snapshot-ready</span>
    <span>Bootloader</span><span class="text-zinc-300">systemd-boot</span>
    <span>Hibernation</span><span class="text-zinc-300">{c.hibernate ? "on (battery detected)" : "off (no battery)"}</span>
    <span>Drivers</span><span class="text-zinc-300">matched to this hardware</span>
    <span>Updates</span><span class="text-zinc-300">one command, signed, everything included</span>
  </div>
</div>

<button class="mt-8 rounded-xl bg-red-600 px-6 py-3 text-sm font-semibold text-white hover:bg-red-500"
        onclick={() => step.update((n) => n + 1)}>
  Erase {c.disk?.path} and install ewe
</button>
