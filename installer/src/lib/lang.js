// Language first (ewe-os 0.17). The Language screen lists languages by
// their own name and the English one ("ქართული — Georgian"), not locale
// codes; picking one suggests everything that follows from it — the
// region (ll_CC), the keyboard, the regional formats, a spelling
// dictionary when Arch has one, and an input method for the languages that
// need one. Every suggestion stays changeable.
//
// Names: the webview's Intl.DisplayNames first; glibc's own locale sources
// (`languages` in the backend: lang_name / language / country_name /
// territory) when the live ISO's WebKit has no data; the code last.

// ── names ──────────────────────────────────────────────────────────────────

const dnCache = new Map();
function displayNames(locale, type) {
  const k = `${locale}|${type}`;
  if (!dnCache.has(k)) {
    let dn = null;
    try {
      if (typeof Intl !== "undefined" && Intl.DisplayNames) dn = new Intl.DisplayNames([locale], { type, fallback: "none" });
    } catch {
      dn = null;
    }
    dnCache.set(k, dn);
  }
  return dnCache.get(k);
}

/** Intl's name for `code`, or "" when it has none (never the code echoed back). */
function intlName(inLocale, type, code) {
  const dn = displayNames(inLocale, type);
  if (!dn || !code) return "";
  try {
    const n = dn.of(code);
    return n && n.toLowerCase() !== code.toLowerCase() ? n : "";
  } catch {
    return "";
  }
}

/** "español" → "Español"; scripts without case are left alone. */
export function capitalize(s, locale) {
  if (!s) return s;
  const chars = Array.from(s);
  let first = chars[0];
  try { first = first.toLocaleUpperCase(locale); } catch { first = first.toUpperCase(); }
  return first + chars.slice(1).join("");
}

const MODIFIERS = {
  latin: "Latin script", cyrillic: "Cyrillic script", devanagari: "Devanagari script",
  valencia: "Valencian", euro: "Euro", saaho: "Saaho", abegede: "Abegede", iqtelif: "Latin script",
};

/** The region of one locale row in English: "Georgia", "Serbia — Latin script". */
export function regionLabel(row) {
  const base = intlName("en", "region", row.country) || row.territory || row.country || "";
  const m = row.modifier ? (MODIFIERS[row.modifier] || row.modifier) : "";
  return m ? (base ? `${base} — ${m}` : m) : base;
}

/** The same region in the language itself ("საქართველო"), or "". */
export function regionNative(row) {
  return intlName(row.lang, "region", row.country) || row.country_name || "";
}

// ── the language list ──────────────────────────────────────────────────────

// When a language is spoken in several regions and none is named after the
// language itself (de_DE, ka_GE), this is where it starts.
const DEFAULT_REGION = {
  en: "US", zh: "CN", pt: "BR", ar: "EG", sv: "SE", ko: "KR", ja: "JP", ka: "GE", vi: "VN",
  hi: "IN", el: "GR", he: "IL", uk: "UA", cs: "CZ", da: "DK", nb: "NO", nn: "NO", fa: "IR",
  ur: "PK", bn: "BD", ta: "IN", te: "IN", sl: "SI", et: "EE", sq: "AL", ms: "MY", sr: "RS",
  ga: "IE", ca: "ES", eu: "ES", gl: "ES", sw: "KE", kk: "KZ", hy: "AM", be: "BY", ne: "NP",
  si: "LK", km: "KH", lo: "LA", my: "MM", am: "ET", gu: "IN", kn: "IN", ml: "IN", mr: "IN",
  pa: "IN", or: "IN", as: "IN", tl: "PH", fil: "PH", zu: "ZA", xh: "ZA", af: "ZA", yo: "NG",
  ha: "NG", ig: "NG", so: "SO", ps: "AF", ku: "TR", ckb: "IQ", uz: "UZ", tg: "TJ", ky: "KG",
  mn: "MN", bo: "CN", ug: "CN", dz: "BT", ti: "ER", lb: "LU", mt: "MT", is: "IS", fo: "FO",
  cy: "GB", gd: "GB", kw: "GB", br: "FR", oc: "FR", co: "FR", fy: "NL", li: "NL", wa: "BE",
};

/** The default locale row of one language group. */
function defaultRow(lang, rows) {
  const plain = rows.filter((r) => !r.modifier);
  const pool = plain.length ? plain : rows;
  const want = DEFAULT_REGION[lang] || lang.toUpperCase();
  return pool.find((r) => r.country === want) || pool.find((r) => r.locale.includes(".UTF-8")) || pool[0];
}

/**
 * The backend's locale rows → one entry per language:
 * { lang, native, english, rows: [...], def: <default row>, search }
 * sorted by the English name, so the list reads the same for everyone.
 */
export function groupLanguages(rows) {
  const by = new Map();
  for (const r of rows || []) {
    if (!r || !r.lang) continue;
    if (!by.has(r.lang)) by.set(r.lang, []);
    by.get(r.lang).push(r);
  }
  const out = [];
  for (const [lang, list] of by) {
    const def = defaultRow(lang, list);
    const english = intlName("en", "language", lang) || def.language || lang;
    let native = intlName(lang, "language", lang);
    // Intl without data for that language answers in English — glibc's own
    // name is better then
    if ((!native || native === english) && def.lang_name && def.lang_name !== english) native = def.lang_name;
    if (!native) native = english;
    native = capitalize(native, lang);
    const regions = list.map((r) => `${regionLabel(r)} ${regionNative(r)} ${r.locale}`).join(" ");
    out.push({
      lang, native, english, rows: list.slice().sort((a, b) => regionLabel(a).localeCompare(regionLabel(b))), def,
      search: `${native} ${english} ${lang} ${regions}`.toLowerCase(),
    });
  }
  out.sort((a, b) => a.english.localeCompare(b.english, "en"));
  return out;
}

/**
 * Filter the list by what was typed. A language found through one of its
 * regions ("brasil", "pt_br") remembers that region, so picking the row
 * picks it.
 */
export function searchLanguages(groups, query) {
  const q = query.trim().toLowerCase();
  if (!q) return groups.map((g) => ({ g, row: null }));
  const hits = [];
  for (const g of groups) {
    if (!g.search.includes(q)) continue;
    const named = g.native.toLowerCase().includes(q) || g.english.toLowerCase().includes(q) || g.lang === q;
    const row = named ? null : g.rows.find((r) =>
      `${regionLabel(r)} ${regionNative(r)} ${r.locale}`.toLowerCase().includes(q)) || null;
    hits.push({ g, row, rank: g.native.toLowerCase().startsWith(q) || g.english.toLowerCase().startsWith(q) ? 0 : 1 });
  }
  hits.sort((a, b) => a.rank - b.rank);
  return hits;
}

/** "Georgian (Georgia)" — how a locale is named in the Summary and Formats. */
export function localeLabel(row) {
  if (!row) return "";
  const lang = intlName("en", "language", row.lang) || row.language || row.lang;
  const region = regionLabel(row);
  return region ? `${lang} (${region})` : lang;
}

/** BCP 47 for Intl: "ka_GE.UTF-8" → "ka-GE", "sr_RS.UTF-8@latin" → "sr-Latn-RS". */
export function bcp47(locale) {
  const [rest, mod] = String(locale || "").split("@");
  const base = rest.split(".")[0];
  const [l, c] = base.split("_");
  const script = mod === "latin" ? "-Latn" : mod === "cyrillic" ? "-Cyrl" : "";
  return `${l}${script}${c ? `-${c}` : ""}`;
}

/** What a formats choice looks like: "21.11.2026 · 14:05 · 1 234,5". "" without Intl. */
export function formatExample(locale) {
  try {
    const tag = bcp47(locale);
    // a webview without that region's data would show another one's — say nothing then
    if (!Intl.DateTimeFormat.supportedLocalesOf([tag]).length) return "";
    const d = new Date(2026, 10, 21, 14, 5);
    const date = new Intl.DateTimeFormat(tag, { dateStyle: "short" }).format(d);
    const time = new Intl.DateTimeFormat(tag, { timeStyle: "short" }).format(d);
    const num = new Intl.NumberFormat(tag).format(1234.5);
    return `${date} · ${time} · ${num}`;
  } catch {
    return "";
  }
}

// ── keyboards ──────────────────────────────────────────────────────────────

// xkb layouts whose first level does not type Latin letters: a computer
// with one of these also gets US English FIRST, so a username and password
// can be typed (and the greeter accepts them) — Super+Space switches.
// ewe-ime's NON_LATIN without jp/kr/cn/tw, which type Latin by default.
export const NON_LATIN = new Set([
  "ru", "ua", "by", "bg", "mk", "rs", "kz", "kg", "mn", "tj", "uz", "ge", "am", "gr", "il",
  "ara", "iq", "sy", "ir", "af", "pk", "ma", "in", "bd", "np", "lk", "th", "la", "kh", "mm",
  "et", "er", "bt", "mv", "eg",
]);

const US = { c: "us", v: "", n: "English (US)" };

/**
 * The keyboards for a locale: its own layout (the one named after its
 * country first, a layout before a variant), US English first when that
 * layout cannot type Latin letters. With an input method on, the plain
 * Latin keyboard the engine reads (`us`, Japanese and Korean keep theirs).
 */
export function suggestLayouts(row, kbdb, withIme = false) {
  if (!row) return [US];
  const db = kbdb || [];
  const find = (c, v = "") => db.find((k) => k.c === c && k.v === v);
  if (withIme) {
    const base = row.lang === "ja" ? "jp" : row.lang === "ko" ? "kr" : "us";
    return [find(base) || (base === "us" ? US : { c: base, v: "", n: base })];
  }
  const cc = String(row.country || "").toLowerCase();
  const term = row.term || "";
  let pick = null;
  if (term) {
    const cands = db.filter((k) => (k.langs || []).includes(term));
    const score = (k) => (k.c === cc ? 0 : 2) + (k.v === "" ? 0 : 1);
    cands.sort((a, b) => score(a) - score(b));
    if (row.lang === "en" && !cands.some((k) => k.c === cc)) pick = find("us") || US;
    else pick = cands[0] || null;
  }
  if (!pick) return [US];
  if (NON_LATIN.has(pick.c) && pick.c !== "us") return [find("us") || US, pick];
  return [pick];
}

/** The parallel lists Hyprland and the helper take: ["us,ge", ""] or ["us,ge", ",qwerty"]. */
export function layoutStrings(list) {
  const l = (list || []).map((k) => k.c).join(",");
  const anyVar = (list || []).some((k) => k.v);
  return [l, anyVar ? list.map((k) => k.v || "").join(",") : ""];
}

// ── typing in other languages (ewe-ime's catalogue) ────────────────────────

const IME_LANG = { pinyin: "zh", mozc: "ja", hangul: "ko", unikey: "vi" };

/** The catalogue's engines for one language code (m17n ids carry theirs). */
export function enginesFor(lang, catalog) {
  return (catalog || []).filter((e) => {
    const m = /^m17n_([a-z]{2,3})_/.exec(e.id);
    return (m ? m[1] : IME_LANG[e.id]) === lang;
  });
}

/** The packages a set of engines needs (fcitx5 itself + each engine's). */
export function imePackages(ids, catalog) {
  const out = [];
  for (const id of ids || []) {
    const e = (catalog || []).find((x) => x.id === id);
    for (const p of (e?.packages || [e?.package]).filter(Boolean)) if (!out.includes(p)) out.push(p);
  }
  return out;
}
