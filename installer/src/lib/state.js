// The installer's single source of truth: every screen writes ONE decision
// here, the Summary restates all of them, and only Install consumes them.
import { writable } from "svelte/store";

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
});

export const step = writable(0);

// icon: Lucide codepoints — the DE's one icon language
export const STEPS = [
  { key: "welcome", label: "Welcome", icon: 0xE1D7 },
  { key: "network", label: "Network", icon: 0xE1AE },
  { key: "timeplace", label: "Time & place", icon: 0xE0E8 },
  { key: "disk", label: "Disk", icon: 0xE0ED },
  { key: "user", label: "Your account", icon: 0xE461 },
  { key: "summary", label: "Summary", icon: 0xE1D0 },
  { key: "install", label: "Install", icon: 0xE0B2 },
];
