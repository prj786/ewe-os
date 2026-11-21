// The installer's single source of truth: every screen writes ONE decision
// here, the Summary restates all of them, and only Install consumes them.
import { writable, derived } from "svelte/store";

export const choices = writable({
  // ── language first (ewe-os 0.17): the Language screen picks a language and
  // its region; everything below follows from it and stays changeable.
  // en_US.UTF-8 is always generated next to the chosen locale. ──
  locale: "en_US.UTF-8",
  langCode: "en",          // the picked language ("ka"); its locale row is `locale`
  formats: "en_US.UTF-8",  // LC_TIME, LC_NUMERIC, LC_MONETARY, LC_MEASUREMENT, LC_PAPER
  // the keyboards, in order — the first is the one the computer starts with
  // (greeter, desktop, console); Super+Space switches. {c: layout, v: variant, n: name}
  layouts: [{ c: "us", v: "", n: "English (US)" }],
  layoutsTouched: false,   // edited on the Keyboard screen: a new language re-suggests anyway
  dict: null,              // {name, package}: an official spelling dictionary to install, or null
  ime: [],                 // ewe-ime engine ids ("mozc"): fcitx5 + these, on at first sign-in
  // ── apps for day one (Komble's curated list): repo apps install with the
  // system; AUR and AppImage apps wait in Komble ("Apps you picked while
  // installing ewe") — nothing is ever built from the AUR in the installer ──
  apps: [],                // [{pkg, name, source: "repo" | "aur" | "appimage"}]
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

// What the Language and Keyboard screens offer, read once from the live
// system: locale rows (`languages`), xkb layouts + variants with their
// languages (`keyboard_db`), ewe-ime's input methods (`ime_catalog`).
// null = still loading.
export const langData = writable({ rows: null, kbdb: null, ime: null });

// The first-party plugin catalogue the live system's ewe payload carries:
// null until the backend answered, [] when the payload has none (an older
// ewe on the ISO) — then the Plugins step is not shown at all.
export const addonsCatalog = writable(null);

// icon: Lucide codepoints — the DE's one icon language
export const STEPS = [
  { key: "language", label: "Language", icon: 0xE0FE },
  { key: "keyboard", label: "Keyboard", icon: 0xE284 },
  { key: "network", label: "Network", icon: 0xE1AE },
  { key: "timeplace", label: "Time & place", icon: 0xE087 },
  { key: "disk", label: "Disk", icon: 0xE0ED },
  { key: "user", label: "Your login", icon: 0xE461 },
  { key: "cloud", label: "Your ewe account", icon: 0xE088 },
  { key: "addons", label: "Plugins", icon: 0xE29C },
  { key: "apps", label: "Apps", icon: 0xE3E4 },
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
