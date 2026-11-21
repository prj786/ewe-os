<script>
  // Apps for day one (optional) — Komble's curated list (src/lib/curated.js,
  // a verbatim copy: scripts/sync-curated.sh). Apps from the Arch
  // repositories install with the system, in the same run. Apps from the AUR
  // and AppImages are never built or fetched here: they wait in Komble after
  // the first sign-in ("Apps you picked while installing ewe"), and every AUR
  // one still goes through Komble's PKGBUILD review. Apps ewe already ships
  // are left out. Nothing picked is a fine answer.
  import { choices } from "../state.js";
  import { curatedCategories } from "../curated.js";

  // ewe ships these already (curated marks them "Ships with ewe")
  const shipped = (a) => /ships with ewe/i.test(a.note || "");
  const cats = curatedCategories
    .map((c) => ({ ...c, apps: c.apps.filter((a) => !shipped(a)) }))
    .filter((c) => c.apps.length);

  let filter = "all";
  $: shown = filter === "all" ? cats : cats.filter((c) => c.id === filter);
  $: picked = new Set($choices.apps.map((a) => a.pkg));
  $: later = $choices.apps.filter((a) => a.source !== "repo");

  function toggle(a) {
    if (picked.has(a.pkg)) $choices.apps = $choices.apps.filter((x) => x.pkg !== a.pkg);
    else $choices.apps = [...$choices.apps, { pkg: a.pkg, name: a.name, source: a.source }];
  }
  const initial = (n) => Array.from(n)[0].toUpperCase();
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Apps</h1>
<p class="mb-4 max-w-xl text-sm text-muted-foreground">
  Pick a few apps to have from the first day, or none. The ones from Arch install with the system; the ones marked
  <em>Komble</em> wait in Komble when you first sign in, one click each.
</p>

<div class="mb-4 flex max-w-2xl flex-wrap gap-1.5" role="tablist" aria-label="Categories">
  <button class="rounded-full border px-3 py-1 text-xs transition-colors {filter === 'all' ? 'border-[var(--accent)] text-foreground' : 'border-border text-muted-foreground hover:bg-muted/60'}"
          role="tab" aria-selected={filter === "all"} onclick={() => (filter = "all")}>All</button>
  {#each cats as c (c.id)}
    <button class="rounded-full border px-3 py-1 text-xs transition-colors {filter === c.id ? 'border-[var(--accent)] text-foreground' : 'border-border text-muted-foreground hover:bg-muted/60'}"
            role="tab" aria-selected={filter === c.id} onclick={() => (filter = c.id)}>{c.title}</button>
  {/each}
</div>

<div class="max-w-2xl">
  {#each shown as c (c.id)}
    <div class="mb-2 mt-4 text-xs font-semibold uppercase tracking-wider text-muted-foreground first:mt-0">{c.title}</div>
    <div class="grid grid-cols-2 gap-2 lg:grid-cols-3">
      {#each c.apps as a (a.pkg)}
        {@const on = picked.has(a.pkg)}
        <button class="flex items-start gap-3 rounded-xl border px-3 py-2.5 text-left transition-colors
                       {on ? 'border-[var(--accent)] bg-card' : 'border-border hover:bg-muted/60'}"
                role="checkbox" aria-checked={on} onclick={() => toggle(a)}>
          <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-muted text-sm font-semibold text-foreground">{initial(a.name)}</span>
          <span class="min-w-0 flex-1">
            <span class="flex items-center gap-1.5">
              <span class="truncate text-sm font-medium">{a.name}</span>
              {#if a.source !== "repo"}
                <span class="shrink-0 rounded px-1 text-[10px] text-muted-foreground ring-1 ring-border">Komble</span>
              {/if}
            </span>
            <span class="line-clamp-2 block text-xs text-muted-foreground">{a.desc}</span>
          </span>
          <span class="icon flex h-5 w-5 shrink-0 items-center justify-center rounded-md border text-[14px] text-primary-foreground"
                style={on ? "background: var(--accent); border-color: var(--accent)" : "border-color: var(--border)"}>
            {on ? String.fromCodePoint(0xE06C) : ""}
          </span>
        </button>
      {/each}
    </div>
  {/each}
</div>

<p class="mt-4 max-w-xl text-xs text-muted-foreground">
  {$choices.apps.length === 0 ? "Nothing selected." : `${$choices.apps.length} selected${later.length ? `, ${later.length} of them in Komble after sign-in` : ""}.`}
  Komble has these and many more any time.
</p>
