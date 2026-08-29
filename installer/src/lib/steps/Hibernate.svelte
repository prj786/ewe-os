<script>
  import { choices } from "../state.js";
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Hibernation</h1>
<p class="mb-6 text-sm text-zinc-400">A swapfile sized to your memory lets the machine power off with your session saved (suspend-then-hibernate on laptops). Costs disk space equal to RAM + 2 GB.</p>

<label class="flex w-full max-w-xl cursor-pointer items-center justify-between rounded-xl border border-zinc-800 px-4 py-3">
  <div>
    <div class="text-sm font-medium">Enable hibernation</div>
    <div class="text-xs text-zinc-500">recommended on laptops — a closed lid never drains to empty</div>
  </div>
  <input type="checkbox" class="h-5 w-5 accent-[var(--accent,#0a84ff)]" bind:checked={$choices.hibernate} />
</label>
