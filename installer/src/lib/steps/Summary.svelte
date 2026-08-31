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
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Time & place</span><span>{c.timezone} · {c.locale}</span></div>
  <div class="flex justify-between border-b border-zinc-800 pb-2"><span class="text-zinc-500">Account</span><span>{c.realName} ({c.username}) on “{c.hostname}”</span></div>
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
