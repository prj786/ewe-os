// The Install log follows its tail (src/lib/follow.js) — the rule, without a
// browser: a stand-in element with the four things the action touches
// (scrollTop / scrollHeight / clientHeight, scroll listeners) and a
// MutationObserver the test fires by hand.
//
//   node installer/tests/follow-test.mjs
//
// What it cannot show is a real engine laying the box out — that is the
// installer's own screen, in the VM pass.
import { strict as assert } from "node:assert";

const observers = [];
globalThis.MutationObserver = class {
  constructor(cb) { this.cb = cb; this.live = false; observers.push(this); }
  observe(node, opts) { this.node = node; this.opts = opts; this.live = true; }
  disconnect() { this.live = false; }
};

const { followTail } = await import("../src/lib/follow.js");

// a 288px box (the screen's h-72) holding `lines` rows of 16px
function box() {
  const listeners = new Set();
  const el = {
    clientHeight: 288,
    lines: 0,
    _top: 0,
    get scrollHeight() { return Math.max(this.clientHeight, this.lines * 16); },
    get scrollTop() { return this._top; },
    // an engine clamps the value and tells listeners when it moved
    set scrollTop(v) {
      const max = this.scrollHeight - this.clientHeight;
      const next = Math.max(0, Math.min(v, max));
      if (next !== this._top) { this._top = next; for (const l of listeners) l(); }
    },
    addEventListener(type, l) { assert.equal(type, "scroll"); listeners.add(l); },
    removeEventListener(type, l) { listeners.delete(l); },
    listeners,
  };
  return el;
}
const mutate = (el, lines) => {         // the helper printed more lines
  el.lines = lines;
  for (const o of observers) if (o.live && o.node === el) o.cb();
};
const atEnd = (el) => el.scrollTop === el.scrollHeight - el.clientHeight;

let n = 0;
const ok = (name) => { n++; console.log(`ok    ${name}`); };

// 1 · lines arrive: the end stays in view
{
  const el = box();
  const action = followTail(el);
  assert.equal(observers.at(-1).opts.childList, true);
  assert.equal(observers.at(-1).opts.subtree, true);
  mutate(el, 10);                       // fits: nothing to scroll
  assert.equal(el.scrollTop, 0);
  mutate(el, 30);                       // 480px of lines in a 288px box
  assert.ok(atEnd(el), `not at the end: ${el.scrollTop}`);
  mutate(el, 400);
  assert.ok(atEnd(el));
  ok("new lines keep the last one in view");

  // 2 · the reader scrolls up: the box stays where they put it
  el.scrollTop = 1000;
  mutate(el, 420);
  assert.equal(el.scrollTop, 1000);
  ok("scrolled up to read: new lines do not pull the view away");

  // 3 · …and back to the end: following resumes
  el.scrollTop = el.scrollHeight;       // clamps to the end, as a real box does
  mutate(el, 440);
  assert.ok(atEnd(el));
  ok("scrolled back to the end: following resumes");

  // 4 · "at the end" is forgiving: a line and a half short still follows
  el.scrollTop = el.scrollHeight - el.clientHeight - 24;
  mutate(el, 460);
  assert.ok(atEnd(el));
  ok("within 24px of the end still counts as the end");

  // 5 · the cap: old lines drop off the top while new ones arrive (the log
  // keeps 400) — the height stops growing, the view stays on the last line
  mutate(el, 460);
  assert.ok(atEnd(el));
  ok("at the 400-line cap the view stays on the last line");

  // 6 · gone with the screen
  action.destroy();
  assert.equal(observers.at(-1).live, false);
  assert.equal(el.listeners.size, 0);
  el.scrollTop = 0;
  mutate(el, 500);
  assert.equal(el.scrollTop, 0);
  ok("destroy() stops observing and listening");
}

// 7 · a box that already holds lines when the screen appears (Retry)
{
  const el = box();
  el.lines = 100;
  followTail(el);
  assert.ok(atEnd(el));
  ok("a log that is already long starts at its end");
}

console.log(`all good (${n} checks)`);
