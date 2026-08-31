// The installer's single source of truth: every screen writes ONE decision
// here, the Summary restates all of them, and only Install consumes them.
import { writable } from "svelte/store";

export const choices = writable({
  locale: "en_US.UTF-8",
  keyboard: "us",
  timezone: "",            // filled by the (confirmed) suggestion or the picker
  tzConfident: false,
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

// icon: Phosphor-Fill codepoints — the DE's one icon language
export const STEPS = [
  { key: "welcome", label: "Welcome", icon: 0xe580 },
  { key: "timeplace", label: "Time & place", icon: 0xe28c },
  { key: "disk", label: "Disk", icon: 0xe2a0 },
  { key: "user", label: "Your account", icon: 0xe4c4 },
  { key: "summary", label: "Summary", icon: 0xeadc },
  { key: "install", label: "Install", icon: 0xe20c },
];
