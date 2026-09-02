<script>
  // Drives the helper verbs in order, streaming its JSON progress. One verb
  // fails → the run stops and says so; the log is always visible. The disk
  // is only touched after a last connectivity check: pacstrap and the ewe
  // layer download everything, and an erased disk with no network behind it
  // was the first bare-metal failure (2026-09-02).
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { choices, step } from "../state.js";

  let log = [];
  let phase = "starting";
  let failed = "";
  let done = false;
  let touched = false;    // a verb that writes to the disk has run
  let running = false;

  function push(l) { log = [...log.slice(-400), l]; }

  async function run() {
    running = true;
    failed = ""; done = false; touched = false; phase = "checking the connection";
    log = [];
    const un = await listen("install-progress", (e) => {
      const p = e.payload;
      if (p.phase) phase = p.phase;
      push(p.msg || p.log || JSON.stringify(p));
    });
    try {
      let net = { connectivity: "unknown" };
      try { net = await invoke("net_status"); } catch {}
      if (net.connectivity !== "full") {
        $choices.online = false;
        throw new Error("Lost the connection — go back to Network and reconnect. Nothing was written to the disk.");
      }
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
        // a full -Syu in the target so first boot owes nothing; before
        // bootloader, which reads the final kernel + microcode
        ["upgrade", [], null],
        ["bootloader", [], null],
      ];
      for (const [stepName, args, secret] of seq) {
        phase = stepName;
        touched = true;
        await invoke("run_step", { step: stepName, args, secret });
      }
      done = true;
      phase = "done";
    } catch (e) {
      failed = e instanceof Error ? e.message : String(e);
      push(`!! ${failed}`);
    }
    un();
    running = false;
  }

  onMount(run);
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">
  {done ? "Done — reboot into ewe" : failed ? "Install failed" : "Installing…"}
</h1>
<p class="mb-4 max-w-2xl text-sm {failed ? 'text-red-400' : 'text-zinc-400'}">
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
{:else if failed && !running}
  <div class="mt-6 flex max-w-2xl items-center gap-3">
    <button class="btn-ghost px-5" onclick={() => step.set(touched ? 5 : 1)}>{touched ? "Back to summary" : "Back to Network"}</button>
    <button class="btn-primary px-5" onclick={run}>Retry</button>
    <span class="text-xs text-zinc-500">
      {touched ? `Retrying starts over: ${$choices.disk?.path} is partitioned again from scratch.` : "Nothing was erased."}
    </span>
  </div>
{/if}
