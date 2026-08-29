<script>
  import { choices } from "../state.js";
  const KEYBOARDS = ["us", "uk", "de", "fr", "es", "it", "ge", "ru", "gr"];
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Welcome</h1>
<p class="mb-6 text-sm text-zinc-400">A few choices and ewe installs itself. Nothing touches your disk until the summary screen.</p>

<label class="mb-1 block text-sm text-zinc-300" for="kb">Keyboard layout</label>
<select id="kb" class="mb-3 w-72 rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm text-zinc-100"
        bind:value={$choices.keyboard}>
  {#each KEYBOARDS as k}<option value={k}>{k}</option>{/each}
</select>
<input class="block w-72 rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm text-zinc-100"
       placeholder="type here to test the layout" />
