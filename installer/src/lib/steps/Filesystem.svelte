<script>
  import { choices } from "../state.js";
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Filesystem</h1>
<p class="mb-6 text-sm text-zinc-400">Both are safe defaults — btrfs adds compression and makes future snapshot/rollback features possible.</p>

{#each [["btrfs", "btrfs", "compression + subvolumes — the ewe default, snapshot-ready"], ["ext4", "ext4", "the classic — simplest possible layout"]] as [val, name, sub]}
  <button class="mb-2 flex w-full max-w-xl flex-col rounded-xl border px-4 py-3 text-left
                 {$choices.fs === val ? 'border-[var(--accent,#0a84ff)] bg-zinc-900' : 'border-zinc-800 hover:bg-zinc-900'}"
          onclick={() => ($choices.fs = val)}>
    <span class="text-sm font-medium">{name}</span>
    <span class="text-xs text-zinc-500">{sub}</span>
  </button>
{/each}
