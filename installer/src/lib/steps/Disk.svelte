<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { choices } from "../state.js";
  let disks = [], battery = false;
  onMount(async () => {
    const p = await invoke("probe");
    disks = p.disks;
    battery = p.battery;
    if ($choices.hibernate === null) $choices.hibernate = battery;
    if (disks.length === 1 && !$choices.disk)
      $choices.disk = { path: disks[0].path, size: disks[0].size, model: disks[0].model };
  });
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Disk</h1>
<p class="mb-6 text-sm text-zinc-400">ewe takes the whole disk. <span class="text-red-400">Everything currently on the chosen disk will be erased</span> — the summary repeats this before anything happens.</p>

{#each disks as d}
  <button class="mb-2 flex w-full max-w-xl items-center justify-between rounded-xl border px-4 py-3 text-left
                 {$choices.disk?.path === d.path ? 'border-[var(--accent)] bg-zinc-800/60' : 'border-zinc-700/60 hover:bg-zinc-800/60'}"
          onclick={() => ($choices.disk = { path: d.path, size: d.size, model: d.model })}>
    <div>
      <div class="text-sm font-medium">{d.model || d.path}</div>
      <div class="text-xs text-zinc-500">{d.path} · {d.tran || "internal"}</div>
    </div>
    <div class="text-sm text-zinc-300">{d.size}</div>
  </button>
{/each}
{#if disks.length === 0}<p class="text-sm text-zinc-500">No fixed disks found.</p>{/if}
