// Dev mock of the `addons` backend command — the 13 add-ons of ewe 0.25's
// payload, copied from their manifests, so the Add-ons step renders in a
// plain browser (`npm run dev`, open /?mock&step=addons). Only used when the
// page runs OUTSIDE Tauri; the real installer reads the live payload.
export const MOCK_ADDONS = [
  {
    "id": "ewe.cast",
    "name": "Cast to TV",
    "description": "Mirror the screen to a Miracast or Chromecast TV from Quick settings, through the ewe-castd engine.",
    "icon": "icCast",
    "category": "Connections",
    "version": "1.0.0"
  },
  {
    "id": "ewe.clipboard",
    "name": "Clipboard & emoji",
    "description": "Clipboard history and an emoji picker under a scissors icon in the bar; records copies, never passwords.",
    "icon": "icClipboard",
    "category": "Desktop",
    "version": "1.1.2"
  },
  {
    "id": "ewe.dock",
    "name": "Dock",
    "description": "A centred bottom dock on the main screen: app launcher, Overview, Komble, the Pen, one box per workspace with its windows, and the dock items other add-ons provide.",
    "icon": "icApps",
    "category": "Desktop",
    "version": "1.0.1"
  },
  {
    "id": "ewe.insomnia",
    "name": "Insomnia",
    "description": "Keeps the screen awake and stops sleep until you turn it off.",
    "icon": "icEye",
    "category": "System",
    "version": "1.0.0"
  },
  {
    "id": "ewe.mail",
    "name": "Mail",
    "description": "Unread mail from your IMAP account or Gmail: an Inbox page in Quick settings, an envelope with the count in the bar, and a notification for new mail.",
    "icon": "icMail",
    "category": "Communication",
    "version": "1.0.0"
  },
  {
    "id": "ewe.media",
    "name": "Music",
    "description": "Now playing: artwork, controls and a seek bar for any MPRIS player, above the dock or under the bar.",
    "icon": "icMusic",
    "category": "Media",
    "version": "1.0.1"
  },
  {
    "id": "ewe.passwords",
    "name": "Passwords",
    "description": "Fill a login from your password manager (1Password, Bitwarden, pass) into any window — Super+P.",
    "icon": "icLock",
    "category": "Security",
    "version": "1.1.1"
  },
  {
    "id": "ewe.phone",
    "name": "Phone",
    "description": "Your phone through KDE Connect: pairing, battery, its notifications and your SMS conversations on the Mobile page, a phone glyph in the bar while it is connected.",
    "icon": "icPhone",
    "category": "Connections",
    "version": "1.0.0"
  },
  {
    "id": "ewe.places",
    "name": "Places",
    "description": "A compact file browser above the dock or under the bar: your home, pinned folders, drag any file out into another app.",
    "icon": "icFolder",
    "category": "Files",
    "version": "1.0.1"
  },
  {
    "id": "ewe.screenshot",
    "name": "Screenshot",
    "description": "A camera in the bar and the Print keys: region, screen or window to ~/Pictures/Screenshots, with a draggable preview stack.",
    "icon": "icCamera",
    "category": "Desktop",
    "version": "1.1.1"
  },
  {
    "id": "ewe.ssh",
    "name": "SSH",
    "description": "Your ~/.ssh/config hosts in Quick settings: a terminal in one click, a SOCKS tunnel with a saved browse script per host, and a glyph in the bar while a tunnel is up.",
    "icon": "icSsh",
    "category": "Network",
    "version": "1.0.0"
  },
  {
    "id": "ewe.sysmon",
    "name": "System monitor",
    "description": "CPU and memory meters in Quick settings, and a compact readout in the top bar if you want one.",
    "icon": "icCpu",
    "category": "System",
    "version": "1.0.0"
  },
  {
    "id": "ewe.vpn",
    "name": "VPN",
    "description": "Your NetworkManager VPN connections in Quick settings: a toggle tile, a list with a one-time sign-in form, and a shield in the bar while one is up.",
    "icon": "icVpn",
    "category": "Network",
    "version": "1.0.0"
  }
];
