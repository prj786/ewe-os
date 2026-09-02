<script>
  // The install downloads the whole system, so the network is a step, not
  // an assumption: nothing past this screen is reachable until
  // NetworkManager reports full connectivity. Wired just works; Wi-Fi is
  // joined right here (nmcli, unprivileged — an active session may).
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { choices } from "../state.js";

  let status = { connectivity: "unknown", wired: false, wifi_device: true, ssid: "" };
  let nets = [];
  let picked = "";          // SSID whose password field is open
  let password = "";
  let joining = "";
  let error = "";
  let scanning = false;
  let timer;

  async function refreshStatus() {
    try {
      status = await invoke("net_status");
      $choices.online = status.connectivity === "full";
    } catch (e) {
      status = { ...status, connectivity: "unknown" };
      $choices.online = false;
    }
  }
  async function refreshList(rescan = false) {
    if (scanning) return;
    scanning = true;
    try { nets = await invoke("wifi_list", { rescan }); }
    catch (e) { error = String(e); }
    scanning = false;
  }
  async function join(n) {
    if (n.secured && picked !== n.ssid) { picked = n.ssid; password = ""; error = ""; return; }
    joining = n.ssid; error = "";
    try {
      await invoke("wifi_connect", { ssid: n.ssid, password: n.secured ? password : null });
      picked = ""; password = "";
      await refreshStatus();
      await refreshList();
    } catch (e) {
      error = String(e);
    }
    joining = "";
  }

  onMount(async () => {
    await refreshStatus();
    await refreshList(true);
    let tick = 0;
    timer = setInterval(async () => {
      await refreshStatus();
      if (++tick % 3 === 0) await refreshList();   // list every 15 s, status every 5
    }, 5000);
  });
  onDestroy(() => clearInterval(timer));

  $: online = status.connectivity === "full";
  $: headline = online
    ? (status.wired ? "Connected — wired" : `Connected to ${status.ssid || "Wi-Fi"}`)
    : status.connectivity === "portal" ? "This network wants a sign-in page"
    : status.connectivity === "limited" ? "Connected, but no internet"
    : "Not connected";
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Network</h1>
<p class="mb-6 text-sm text-zinc-400">ewe downloads itself during the install, so it needs the internet before any disk is touched. Plug in a cable or join a Wi-Fi network.</p>

<div class="card mb-6 flex max-w-xl items-center gap-3 p-4">
  <span class="ph-i text-[22px]" style={online ? "color: var(--accent)" : "color: var(--color-zinc-500)"}>
    {String.fromCodePoint(online ? 0xe182 : status.wired ? 0xe4ea : 0xebc0)}
  </span>
  <div class="min-w-0 flex-1">
    <div class="text-sm font-medium">{headline}</div>
    <div class="text-xs text-zinc-500">
      {online ? "You can continue." : status.connectivity === "portal" ? "Open a browser from the dock to sign in, then come back." : "Waiting for a connection…"}
    </div>
  </div>
</div>

{#if status.wifi_device}
  <div class="mb-2 flex max-w-xl items-center justify-between">
    <span class="text-sm text-zinc-300">Wi-Fi networks</span>
    <button class="btn-ghost px-3 py-1 text-xs" disabled={scanning} onclick={() => refreshList(true)}>{scanning ? "scanning…" : "Refresh"}</button>
  </div>
  {#each nets as n (n.ssid)}
    <div class="mb-2 max-w-xl rounded-xl border {n.in_use ? 'border-[var(--accent)] bg-zinc-800/60' : 'border-zinc-700/60'}">
      <button class="flex w-full items-center justify-between px-4 py-3 text-left hover:bg-zinc-800/60 rounded-xl"
              disabled={joining !== ""} onclick={() => join(n)}>
        <div class="flex items-center gap-3">
          <span class="ph-i text-[16px] text-zinc-400">{String.fromCodePoint(n.signal > 66 ? 0xebc4 : n.signal > 33 ? 0xebc2 : 0xebc1)}</span>
          <span class="text-sm font-medium">{n.ssid}</span>
          {#if n.secured}<span class="ph-i text-[12px] text-zinc-500">{String.fromCodePoint(0xe3c6)}</span>{/if}
        </div>
        <span class="text-xs text-zinc-500">{n.in_use ? "connected" : joining === n.ssid ? "joining…" : `${n.signal}%`}</span>
      </button>
      {#if picked === n.ssid && !n.in_use}
        <form class="flex items-center gap-2 border-t border-zinc-800 px-4 py-3" onsubmit={(e) => { e.preventDefault(); join(n); }}>
          <input class="input flex-1" type="password" placeholder="password" bind:value={password} />
          <button class="btn-primary px-4" type="submit" disabled={password.length < 8 || joining !== ""}>Join</button>
        </form>
      {/if}
    </div>
  {/each}
  {#if nets.length === 0 && !scanning}<p class="text-sm text-zinc-500">No networks seen yet.</p>{/if}
{:else}
  <p class="max-w-xl text-sm text-zinc-500">No Wi-Fi adapter found — connect an ethernet cable; the status above updates by itself.</p>
{/if}

{#if error}<p class="mt-3 max-w-xl text-sm text-red-400">{error}</p>{/if}
