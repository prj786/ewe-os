// The first-party plugin catalogue, as the installer shows it (ewe 0.25+,
// they are opt-in — docs/PLUGINS.md "First-party plugins"; called "add-ons"
// before 0.25.1, hence the file and identifier names). The backend reads the LIVE system's
// payload (/usr/share/ewe/plugins/bundle.json + each manifest.json); this
// file turns the manifests' Theme icon NAMES into Lucide glyphs and groups
// the rows the way the Welcome screen and Komble present them.

// manifest `icon` is a Theme property name ("icMusic"); the shell resolves it
// as Theme[icon]. The installer bundles the same Lucide face and codepoints
// the shell loads (app.css), so the mapping is Theme.qml's table, verbatim.
const GLYPHS = {
  icApps: 0xe0e9,       // grid-3x3 — the dock
  icCamera: 0xe064,     // camera — screenshot
  icCast: 0xe066,       // cast — cast to TV
  icClipboard: 0xe14e,  // scissors — clipboard
  icCpu: 0xe0a9,        // cpu — system monitor
  icEye: 0xe0ba,        // eye — insomnia
  icFolder: 0xe0d7,     // folder — places
  icLock: 0xe10b,       // lock — passwords
  icMail: 0xe10f,       // mail
  icMusic: 0xe122,      // music
  icPhone: 0xe163,      // smartphone — phone
  icSsh: 0xe20a,        // square-terminal — ssh
  icVpn: 0xe1ff,        // shield-check — vpn
};
export const GLYPH_FALLBACK = 0xe29c; // puzzle — a plugin without a known icon
export const GLYPH_CHECK = 0xe06c;    // check — the checkbox tick

export function glyphFor(icon) {
  return String.fromCodePoint(GLYPHS[icon] || GLYPH_FALLBACK);
}

// Categories in the order a newcomer reads them: what you see first (the
// dock, the bar) before what you connect. Unknown categories follow, sorted.
const CATEGORY_ORDER = ["Desktop", "Media", "Files", "System", "Network", "Connections", "Communication", "Security"];

export function groupAddons(list) {
  const groups = new Map();
  for (const a of list || []) {
    const cat = a.category || "Other";
    if (!groups.has(cat)) groups.set(cat, []);
    groups.get(cat).push(a);
  }
  const rank = (c) => {
    const i = CATEGORY_ORDER.indexOf(c);
    return i < 0 ? CATEGORY_ORDER.length : i;
  };
  return [...groups.entries()]
    .sort(([a], [b]) => rank(a) - rank(b) || a.localeCompare(b))
    .map(([category, items]) => ({
      category,
      items: items.slice().sort((x, y) => (x.name || x.id).localeCompare(y.name || y.id)),
    }));
}

// The one row that gets a hint: a fresh ewe has no dock (0.25), and most
// people coming from another desktop expect one. Same words as Welcome.
export const DOCK_ID = "ewe.dock";
export const DOCK_HINT = "Recommended if you like a dock";
