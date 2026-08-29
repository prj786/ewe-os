<script>
  // Drives the helper verbs in order, streaming its JSON progress. One verb
  // fails → the run stops and says so; the log is always visible.
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { choices } from "../state.js";

  let log = [];
  let phase = "starting";
  let failed = "";
  let done = false;

  function push(l) { log = [...log.slice(-400), l]; }

  onMount(async () => {
    const un = await listen("install-progress", (e) => {
      const p = e.payload;
      if (p.phase) phase = p.phase;
      push(p.msg || p.log || JSON.stringify(p));
    });
    const c = $choices;
    const erase = `ERASE=${c.disk.path}`;
    const seq = [
      ["partition", [c.disk.path, c.fs, erase], null],
      ["mkfs", [c.disk.path, c.fs, erase], null],
      ["pacstrap", [], null],
      ...(c.hibernate ? [["hibernate", [c.fs], null]] : []),
      ["settz", [c.timezone], null],
      ["setlocale", [c.locale], null],
      ["sethostname", [c.hostname], null],
      ["user", [c.realName, c.username], c.password],
      ["layer", [], null],
      ["bootloader", [], null],
    ];
    try {
      for (const [stepName, args, secret] of seq) {
        phase = stepName;
        await invoke("run_step", { step: stepName, args, secret });
      }
      done = true;
      phase = "done";
    } catch (e) {
      failed = String(e);
    }
    un();
  });
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">
  {done ? "Done — reboot into ewe" : failed ? "Install failed" : "Installing…"}
</h1>
<p class="mb-4 text-sm {failed ? 'text-red-400' : 'text-zinc-400'}">
  {failed || (done ? "Remove the USB stick and restart. The greeter will be waiting." : `current step: ${phase}`)}
</p>

<div class="h-72 max-w-2xl overflow-y-auto rounded-xl border border-zinc-800 bg-black/40 p-3 font-mono text-xs text-zinc-400">
  {#each log as l}<div>{l}</div>{/each}
</div>

{#if done}
  <button class="mt-6 rounded-xl bg-[var(--accent,#0a84ff)] px-6 py-3 text-sm font-semibold text-white"
          onclick={() => invoke("run_step", { step: "reboot", args: [], secret: null })}>
    Reboot
  </button>
{/if}
