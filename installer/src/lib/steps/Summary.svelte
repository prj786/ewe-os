<script>
  // The only screen with an Install button. Every choice in plain words;
  // the destructive one in red.
  import { choices, step } from "../state.js";
  $: c = $choices;
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Summary</h1>
<p class="mb-6 text-sm text-zinc-400">Read it once — this is the moment before anything is touched.</p>

<div class="max-w-xl space-y-2 text-sm">
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Disk</span><span class="text-red-400">{c.disk?.path} ({c.disk?.size}) — will be completely erased</span></div>
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Filesystem</span><span>{c.fs}{c.fs === "btrfs" ? " (@ / @home, zstd)" : ""}</span></div>
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Hibernation</span><span>{c.hibernate ? "on — swapfile RAM+2G" : "off"}</span></div>
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Time & place</span><span>{c.timezone} · {c.locale}</span></div>
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Account</span><span>{c.realName} ({c.username}) on “{c.hostname}”</span></div>
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Desktop</span><span>ewe, drivers matched to this hardware, updates via pacman</span></div>
</div>

<button class="mt-8 rounded-xl bg-red-600 px-6 py-3 text-sm font-semibold text-white hover:bg-red-500"
        onclick={() => step.update((n) => n + 1)}>
  Erase {c.disk?.path} and install ewe
</button>
