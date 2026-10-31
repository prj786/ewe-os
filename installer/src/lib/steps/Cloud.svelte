<script>
  // Your ewe account (2026-10-31) — sign in or skip. Signing in uses your
  // server's own page in the browser (ewe never sees your Nextcloud
  // password); the installer then reads what the account holds and, if it
  // has a backup, this computer starts with all of it: settings and look,
  // the folders that sync, the apps Komble offers, the plugins you had
  // (pre-picked on the next screen; you can still change them). Skipping is
  // a fine answer: the desktop signs in any time.
  import { onMount, onDestroy } from "svelte";
  import { invoke as tauriInvoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { choices, addonsCatalog } from "../state.js";

  const params = new URLSearchParams(location.search);
  const mock = params.has("mock") && !("__TAURI_INTERNALS__" in window);
  async function call(cmd, args = {}) {
    if (mock) {
      const { mockCloud } = await import("../mock.js");
      return mockCloud(cmd, args, params);
    }
    return tauriInvoke(cmd, args);
  }

  let server = $choices.account?.server || "";
  let url = "";
  let error = "";
  let reading = false;
  let unlisten = null;

  onMount(async () => {
    if (!mock) unlisten = await listen("cloud-login-url", (e) => (url = String(e.payload || "")));
    if (mock && params.has("signed")) {
      $choices.account = { server: "https://cloud.example.org", login: "dolly", name: "Dolly Sheep", email: "", backup: null, restore: true };
      await readBackup();
    }
  });
  onDestroy(() => unlisten && unlisten());

  async function signIn() {
    let s = server.trim();
    if (!s) { error = "Enter your server’s address, then sign in."; return; }
    error = ""; url = "";
    $choices.signingIn = true;
    try {
      const r = await call("cloud_login", { server: s });
      if (r?.ok) {
        $choices.account = {
          server: r.server, login: r.login_name || r.user, name: r.display_name || r.user,
          email: r.email || "", backup: null, restore: true
        };
        await readBackup();
      } else if (r && r.error !== "cancelled") {
        error = r.message || r.error || "Couldn’t sign in. Check the address and try again.";
      }
    } catch (e) {
      error = String(e);
    } finally {
      $choices.signingIn = false;
      url = "";
    }
  }

  async function readBackup() {
    reading = true;
    try {
      const b = await call("cloud_backup");
      $choices.account = { ...$choices.account, backup: b };
      pickFromAccount(b);
    } catch (e) {
      $choices.account = { ...$choices.account, backup: { exists: false, error: String(e) } };
    } finally {
      reading = false;
    }
  }

  // the plugins that computer had, pre-picked on the Plugins screen (only
  // first-party ones this ISO carries; the user can still turn any off)
  function pickFromAccount(b) {
    if (!b?.exists) return;
    const catalog = ($addonsCatalog || []).map((a) => a.id);
    const want = (b.plugins || []).filter((id) => catalog.includes(id));
    $choices.fromAccount = want;
    $choices.addons = catalog.filter((id) => want.includes(id) || $choices.addons.includes(id));
  }

  async function signOut() {
    try { await call("cloud_signout"); } catch {}
    const drop = new Set($choices.fromAccount);
    $choices.addons = $choices.addons.filter((id) => !drop.has(id));
    $choices.fromAccount = [];
    $choices.account = null;
  }

  $: acct = $choices.account;
  $: b = acct?.backup;
  $: when = b?.modified ? new Date(b.modified).toLocaleString([], { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" }) : "";
  $: host = String(acct?.server || "").replace(/^https?:\/\//, "");
  $: fromNames = $choices.fromAccount.map((id) => ($addonsCatalog || []).find((a) => a.id === id)?.name || id);
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Your ewe account</h1>
<p class="mb-6 max-w-xl text-sm text-zinc-400">
  Sign in to your own Nextcloud and this computer starts with everything from your account: your settings and look,
  your folders, your browser and the plugins you use. Or press Next to skip — you can sign in from the desktop any time.
</p>

{#if !acct}
  <div class="max-w-xl">
    <label class="mb-1 block text-sm text-zinc-300" for="srv">Your Nextcloud</label>
    <div class="flex gap-2">
      <input id="srv" class="input w-80" bind:value={server} placeholder="cloud.example.org"
             disabled={$choices.signingIn} onkeydown={(e) => e.key === "Enter" && !$choices.signingIn && signIn()} />
      {#if $choices.signingIn}
        <button class="btn-ghost" onclick={() => call("cloud_cancel")}>Cancel</button>
      {:else}
        <button class="btn-primary px-5" onclick={signIn}>Sign in</button>
      {/if}
    </div>
    {#if $choices.signingIn}
      <p class="mt-3 text-sm text-zinc-400">Waiting for the browser… Finish signing in on your server’s page, then come back here.</p>
      {#if url}
        <button class="mt-1 text-sm underline" style="color: var(--accent)" onclick={() => navigator.clipboard.writeText(url)}>Copy the sign-in link</button>
      {/if}
    {/if}
    {#if error}
      <p class="mt-3 text-sm text-red-400">{error}</p>
    {/if}
    <p class="mt-6 text-xs text-zinc-500">
      Your browser opens your server’s own sign-in page; ewe never sees your Nextcloud password. The server gives this
      computer its own app password, which you can revoke in Nextcloud → Settings → Security.
    </p>
  </div>
{:else}
  <div class="card max-w-xl p-4">
    <div class="flex items-center gap-3">
      <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-base font-semibold text-white"
            style="background: var(--accent)">{(acct.name || acct.login || "?").slice(0, 1).toUpperCase()}</span>
      <span class="min-w-0 flex-1">
        <span class="block text-sm font-medium">{acct.name || acct.login}</span>
        <span class="block truncate text-xs text-zinc-500">{acct.login} on {host}</span>
      </span>
      <button class="btn-ghost text-xs" onclick={signOut}>Use another account</button>
    </div>
  </div>

  <div class="mt-4 max-w-xl">
    {#if reading}
      <p class="text-sm text-zinc-400">Reading what your account holds…</p>
    {:else if b?.exists}
      <p class="mb-3 text-sm text-zinc-300">
        Your account has a backup from <span class="font-medium">“{b.machine || "your other computer"}”</span>{when ? `, saved ${when}` : ""}.
      </p>
      <ul class="mb-4 space-y-1.5 text-sm text-zinc-400">
        <li><span class="text-zinc-200">Settings and look</span> — theme, bar, displays, keyboard, shortcuts</li>
        {#if b.folders?.length}
          <li><span class="text-zinc-200">{b.folders.length} folder{b.folders.length === 1 ? "" : "s"}</span> — {b.folders.join(", ")}; they come down after the first start</li>
        {/if}
        {#if b.apps}
          <li><span class="text-zinc-200">{b.apps} app{b.apps === 1 ? "" : "s"}</span> — waiting in Komble; nothing installs by itself</li>
        {/if}
        {#if fromNames.length}
          <li><span class="text-zinc-200">{fromNames.length} plugin{fromNames.length === 1 ? "" : "s"}</span> — {fromNames.join(", ")}; picked on the next screen</li>
        {/if}
        <li><span class="text-zinc-200">Your browser</span> — bookmarks, history and open tabs, as soon as it starts</li>
      </ul>
      <button class="flex items-center gap-3 text-left text-sm" role="checkbox" aria-checked={acct.restore}
              onclick={() => ($choices.account = { ...acct, restore: !acct.restore })}>
        <span class="icon flex h-5 w-5 shrink-0 items-center justify-center rounded-md border text-[14px] text-white"
              style={acct.restore ? "background: var(--accent); border-color: var(--accent)" : "border-color: var(--color-zinc-500)"}>
          {acct.restore ? String.fromCodePoint(0xe06c) : ""}
        </span>
        <span>Bring all of it to this computer</span>
      </button>
      {#if !acct.restore}
        <p class="mt-2 text-xs text-zinc-500">This computer starts fresh but signed in; the desktop can still restore later.</p>
      {/if}
    {:else}
      <p class="text-sm text-zinc-400">
        {b?.error ? `Couldn’t read your account (${b.error}). ` : "Your account has no backup yet. "}This computer
        starts signed in; its settings become your backup.
      </p>
    {/if}
  </div>
{/if}
