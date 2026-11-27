// Every first-party plugin the payload carries has its own glyph on the
// installer's Plugins screen (src/lib/addons.js) — a plugin whose icon the
// table does not know shows the puzzle piece, which is how Window switcher
// (icFloating) looked until 0.18.
//
//   node installer/tests/glyphs-test.mjs <plugins dir> [<plugins dir> …]
//
// <plugins dir> holds bundle.json and one folder per plugin: an ewe checkout's
// plugins/, or /usr/share/ewe/plugins. The codepoints are checked against the
// DE's own table when a Theme.qml is found beside it (…/dotfiles/quickshell).
import { readFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { glyphFor, GLYPH_FALLBACK } from "../src/lib/addons.js";

const dirs = process.argv.slice(2);
if (!dirs.length) { console.log("usage: node installer/tests/glyphs-test.mjs <plugins dir> …"); process.exit(2); }
let fail = 0;
for (const dir of dirs) {
  const bundle = JSON.parse(readFileSync(join(dir, "bundle.json"), "utf8")).plugins || {};
  const themeFile = join(dirname(dir), "dotfiles", "quickshell", "Theme.qml");
  const theme = existsSync(themeFile) ? readFileSync(themeFile, "utf8") : "";
  let n = 0;
  for (const id of Object.keys(bundle).sort()) {
    let m;
    try { m = JSON.parse(readFileSync(join(dir, id, "manifest.json"), "utf8")); } catch { continue; }
    n++;
    const cp = glyphFor(m.icon).codePointAt(0);
    if (cp === GLYPH_FALLBACK) { console.log(`FAIL  ${id}: no glyph for "${m.icon}" (${m.name})`); fail = 1; continue; }
    // Theme.qml: `readonly property string icMusic: ic(0xE122)`
    const t = new RegExp(`\\b${m.icon}:\\s*ic\\(0x([0-9a-fA-F]+)\\)`).exec(theme);
    if (t && parseInt(t[1], 16) !== cp) {
      console.log(`FAIL  ${id}: ${m.icon} is U+${cp.toString(16)} here, U+${t[1].toLowerCase()} in Theme.qml`); fail = 1;
    }
  }
  console.log(`${fail ? "…   " : "ok   "} ${n} plugins in ${dir}${theme ? " (codepoints checked against Theme.qml)" : ""}`);
}
console.log(fail ? "FAILED" : "all good");
process.exit(fail);
