// The installer's single source of truth: every screen writes ONE decision
// here, the Summary restates all of them, and only Install consumes them.
import { writable, derived } from "svelte/store";

export const choices = writable({
  locale: "en_US.UTF-8",
  keyboard: "us",
  online: false,           // Network step: NetworkManager reports full connectivity
  timezone: "",            // filled by the (confirmed) suggestion or the picker
  tzConfident: false,
  tzConfirmed: false,       // a low-confidence guess needs an explicit click (Use this zone / a pick)
  disk: null,              // { path, size, model }
  // ── decided by ewe, not asked (the product IS the absence of these
  // questions): btrfs @/@home+zstd, systemd-boot, swapfile hibernation
  // on battery machines, stock kernel, drivers by lspci ──
  fs: "btrfs",
  hibernate: null,         // auto: battery presence at probe time
  hostname: "ewe",
  realName: "",
  username: "",
  password: "",
  // ── first-party plugins (ewe 0.25: opt-in, nothing pre-checked; the UI
  // says "Plugins", the key stays `addons`). Ids out of the live payload's
  // bundle.json; installed for the new user after ewe-setup, each one
  // best-effort — a failing plugin never fails the OS install ──
  addons: [],
  // ── the ewe account (2026-10-31): sign in or skip. null = skipped. Never
  // the app password — that stays in the backend (account.rs) and reaches
  // the helper on stdin. { server, login, name, email, backup, restore } —
  // backup: what the account holds (account.rs summarize), restore: bring
  // it to this computer (default yes when there is one) ──
  account: null,
  signingIn: false,        // a sign-in waits for the browser: no Next
  fromAccount: [],         // plugin ids pre-picked because the backup had them
});

export const step = writable(0);

// The first-party plugin catalogue the live system's ewe payload carries:
// null until the backend answered, [] when the payload has none (an older
// ewe on the ISO) — then the Plugins step is not shown at all.
export const addonsCatalog = writable(null);

// icon: Lucide codepoints — the DE's one icon language
export const STEPS = [
  { key: "welcome", label: "Welcome", icon: 0xE1D7 },
  { key: "network", label: "Network", icon: 0xE1AE },
  { key: "timeplace", label: "Time & place", icon: 0xE0E8 },
  { key: "disk", label: "Disk", icon: 0xE0ED },
  { key: "user", label: "Your login", icon: 0xE461 },
  { key: "cloud", label: "Your ewe account", icon: 0xE088 },
  { key: "addons", label: "Plugins", icon: 0xE29C },
  { key: "summary", label: "Summary", icon: 0xE1D0 },
  { key: "install", label: "Install", icon: 0xE0B2 },
];

// The steps actually shown: Plugins only when the payload offers any.
// Everything that navigates by position (the rail, Next, "Back to summary")
// goes through this list and `indexOf(key)`, never a literal index.
export const visibleSteps = derived(addonsCatalog, ($catalog) =>
  STEPS.filter((s) => s.key !== "addons" || ($catalog !== null && $catalog.length > 0)),
);

export function indexOf(steps, key) {
  const i = steps.findIndex((s) => s.key === key);
  return i < 0 ? 0 : i;
}
