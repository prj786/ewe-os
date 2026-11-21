<script>
  // Keyboard — filled in from the language (Language screen), changeable.
  // The list is ordered: the first keyboard is the one the new computer
  // starts with, at the login screen too; Super+Space switches. The choice
  // is applied to this live session at once (App.svelte), so the field below
  // tests the real thing and the password is typed the way the login screen
  // will read it. Layouts and variants come from the live system's xkb
  // registry — the list ewe Settings offers.
  import { choices, langData } from "../state.js";
  import { NON_LATIN } from "../lang.js";

  let query = "";

  $: list = $choices.layouts;
  $: db = $langData.kbdb || [];
  $: matches = query.trim()
    ? db.filter((k) => !list.some((l) => l.c === k.c && l.v === k.v)
                 && `${k.n} ${k.c} ${k.v}`.toLowerCase().includes(query.trim().toLowerCase())).slice(0, 8)
    : [];
  $: latinFirst = list.length > 0 && !NON_LATIN.has(list[0].c);

  function set(l) {
    $choices.layouts = l;
    $choices.layoutsTouched = true;
  }
  const add = (k) => { set([...list, { c: k.c, v: k.v, n: k.n }].slice(0, 4)); query = ""; };
  const remove = (i) => { if (list.length > 1) set(list.filter((_, j) => j !== i)); };
  function up(i) {
    if (i <= 0) return;
    const l = list.slice();
    [l[i - 1], l[i]] = [l[i], l[i - 1]];
    set(l);
  }
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Keyboard</h1>
<p class="mb-6 max-w-xl text-sm text-muted-foreground">
  {list.length > 1
    ? "The first keyboard is the one this computer starts with, at the login screen too. Super+Space switches between them."
    : "The keyboard this computer starts with, at the login screen too. Add another if you type in more than one."}
</p>

<div class="card mb-3 max-w-xl divide-y divide-border">
  {#each list as k, i (`${k.c}:${k.v}`)}
    <div class="flex items-center gap-3 px-4 py-2.5 text-sm">
      <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-muted text-xs text-muted-foreground">{i + 1}</span>
      <span class="min-w-0 flex-1 truncate">{k.n}</span>
      <span class="text-xs text-muted-foreground">{k.c}{k.v ? `(${k.v})` : ""}</span>
      <button class="btn-ghost h-7 w-7 px-0" disabled={i === 0} title="Make this the first one" aria-label="Move {k.n} up"
              onclick={() => up(i)}><span class="icon text-[14px]">{String.fromCodePoint(0xE04A)}</span></button>
      <button class="btn-ghost h-7 w-7 px-0" disabled={list.length === 1} title="Remove" aria-label="Remove {k.n}"
              onclick={() => remove(i)}><span class="icon text-[14px]">{String.fromCodePoint(0xE1B2)}</span></button>
    </div>
  {/each}
</div>

{#if list.length < 4}
  <div class="relative mb-6 max-w-xl">
    <span class="icon pointer-events-none absolute left-3 top-2 text-[14px] text-muted-foreground">{String.fromCodePoint(0xE13D)}</span>
    <input class="input pl-9" placeholder="Add a keyboard: Georgian, German (no dead keys), Dvorak…" bind:value={query}
           aria-label="Add a keyboard" />
    {#if matches.length}
      <div class="card mt-1 divide-y divide-border">
        {#each matches as k (`${k.c}:${k.v}`)}
          <button class="flex w-full items-center justify-between px-4 py-2 text-left text-sm hover:bg-muted/60" onclick={() => add(k)}>
            <span>{k.n}</span><span class="text-xs text-muted-foreground">{k.c}{k.v ? `(${k.v})` : ""}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<label class="mb-1 block text-sm text-muted-foreground" for="try">Try it</label>
<input id="try" class="input max-w-xl" placeholder="Type here to try your keyboard" />
{#if !latinFirst}
  <p class="mt-3 max-w-xl text-xs text-muted-foreground">
    {list[0]?.n} comes first, so your username — letters a to z — needs Super+Space on the next screens.
  </p>
{/if}
