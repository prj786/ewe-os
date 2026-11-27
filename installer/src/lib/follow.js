// followTail — a Svelte action that keeps a scrolling log on its last line.
//
// The Install screen's log is a fixed-height box that the helper's lines are
// appended to for ten minutes. Nothing scrolled it: the box sat on the first
// eighteen lines while everything that happened was added below the fold, so
// an install looked frozen (ewe-os 0.18).
//
// The rule is a terminal's: while the reader is at the end, new lines keep the
// end in view; once they scroll up to read something, the box stays where
// they put it, and following resumes when they scroll back down to the end.
//
// Plain DOM on purpose (no Svelte, no Tauri), so installer/tests/
// follow-test.mjs can run it under node with a stand-in element.

const NEAR_END = 24; // px: "at the end" allows for a partly shown last line

export function followTail(node) {
  let follow = true;
  const atEnd = () => node.scrollHeight - node.scrollTop - node.clientHeight <= NEAR_END;
  // our own scrollTop write lands here too, and reads as "at the end"
  const onScroll = () => { follow = atEnd(); };
  const toEnd = () => { if (follow) node.scrollTop = node.scrollHeight; };
  // lines are added (and, past the cap, dropped from the top) by the keyed
  // {#each}: any change under the box is a reason to look at the end again
  const observer = new MutationObserver(toEnd);
  observer.observe(node, { childList: true, subtree: true, characterData: true });
  node.addEventListener("scroll", onScroll, { passive: true });
  toEnd();
  return {
    destroy() {
      observer.disconnect();
      node.removeEventListener("scroll", onScroll);
    },
  };
}
