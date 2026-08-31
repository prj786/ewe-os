<script>
  import { choices } from "../state.js";
  // username auto-derives from the name until the user edits it directly
  let touched = false;
  $: if (!touched && $choices.realName)
    $choices.username = $choices.realName.toLowerCase().replace(/[^a-z0-9]+/g, "").slice(0, 16);
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Your account</h1>
<p class="mb-6 text-sm text-zinc-400">The first user is an administrator (sudo via password).</p>

<label class="mb-1 block text-sm text-zinc-300" for="rn">Your name</label>
<input id="rn" class="input mb-3 w-72"
       bind:value={$choices.realName} placeholder="Dolly Sheep" />
<label class="mb-1 block text-sm text-zinc-300" for="un">Username</label>
<input id="un" class="input mb-3 w-72"
       bind:value={$choices.username} oninput={() => (touched = true)} placeholder="dolly" />
<label class="mb-1 block text-sm text-zinc-300" for="pw">Password</label>
<input id="pw" type="password" class="input mb-3 w-72"
       bind:value={$choices.password} placeholder="at least 4 characters" />
<label class="mb-1 block text-sm text-zinc-300" for="hn">Computer name</label>
<input id="hn" class="input w-72"
       bind:value={$choices.hostname} />
