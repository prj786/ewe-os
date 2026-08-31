<script>
  // Layouts come from the live system's xkb registry (~99, human names) —
  // the same list the DE's Settings offers, not a hardcoded handful.
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { choices } from "../state.js";
  let layouts = [{ c: "us", n: "English (US)" }];
  onMount(async () => {
    try {
      const r = await invoke("keyboard_layouts");
      if (Array.isArray(r) && r.length) layouts = r;
    } catch {}
  });
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Welcome</h1>
<p class="mb-6 text-sm text-zinc-400">A few choices and ewe installs itself. Nothing touches your disk until the summary screen.</p>

<label class="mb-1 block text-sm text-zinc-300" for="kb">Keyboard layout</label>
<select id="kb" class="input mb-3 w-72" bind:value={$choices.keyboard}>
  {#each layouts as l}<option value={l.c}>{l.n}</option>{/each}
</select>
<input class="input block w-72" placeholder="type here to test the layout" />
