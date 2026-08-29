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

export const STEPS = [
  { key: "welcome", label: "Welcome" },
  { key: "timeplace", label: "Time & place" },
  { key: "disk", label: "Disk" },
  { key: "user", label: "Your account" },
  { key: "summary", label: "Summary" },
  { key: "install", label: "Install" },
];
