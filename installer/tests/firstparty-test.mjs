// ewe's own opt-in apps on the Apps screen (src/lib/firstparty.js): what is
// offered, and that nothing is offered twice.
//
//   node installer/tests/firstparty-test.mjs [<plugins dir>]
//
// With a payload's plugins dir (an ewe checkout's plugins/, or
// /usr/share/ewe/plugins) it also checks the reason ewe Safe is left to the
// Plugins screen: that payload's ewe.safe plugin really requires the package.
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { firstParty, firstPartyApps } from "../src/lib/firstparty.js";

const pkgs = (list) => list.map((a) => a.pkg);
let n = 0;
const ok = (name) => { n++; console.log(`ok    ${name}`); };

// the Plugins screen offers ewe Safe (every payload since ewe 0.35): Mobile only
assert.deepEqual(pkgs(firstPartyApps([{ id: "ewe.dock" }, { id: "ewe.safe" }, { id: "ewe.phone" }])), ["ewe-mobile"]);
ok("with the ewe Safe plugin on the Plugins screen: only ewe Mobile here");

// no such plugin (an older payload), or no Plugins screen at all: both
assert.deepEqual(pkgs(firstPartyApps([{ id: "ewe.dock" }])), ["ewe-mobile", "ewe-safe"]);
assert.deepEqual(pkgs(firstPartyApps([])), ["ewe-mobile", "ewe-safe"]);
assert.deepEqual(pkgs(firstPartyApps(null)), ["ewe-mobile", "ewe-safe"]);
ok("without it: ewe Mobile and ewe Safe");

// the shape the Apps screen and the helper's `apps` verb take: a repo pick
// whose name passes the helper's valid_pkg
for (const a of firstParty) {
  assert.equal(a.source, "repo", a.pkg);
  assert.match(a.pkg, /^[a-z0-9@_+][a-z0-9@._+-]{0,99}$/, a.pkg);
  assert.ok(a.name && a.desc, a.pkg);
}
ok("each is a plain repo pick with a name the helper accepts");

const dir = process.argv[2];
if (dir) {
  const read = (id) => JSON.parse(readFileSync(join(dir, id, "manifest.json"), "utf8"));
  const safe = read("ewe.safe");
  assert.ok((safe.requires?.packages || []).includes("ewe-safe"), "ewe.safe does not require ewe-safe");
  ok(`${dir}: the ewe Safe plugin requires the ewe-safe package (the Plugins screen installs it)`);
  const phone = read("ewe.phone");
  assert.ok(!(phone.requires?.packages || []).includes("ewe-mobile"), "ewe.phone requires ewe-mobile now: offer it once");
  ok(`${dir}: no plugin brings ewe-mobile (the Phone plugin does not require it)`);
}
console.log(`all good (${n} checks)`);
