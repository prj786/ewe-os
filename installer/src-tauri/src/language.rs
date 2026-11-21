//! Language first (ewe-os 0.17): what the Language and Keyboard screens
//! offer, read from the LIVE system so the installer shows exactly what the
//! installed one will have.
//!
//! - `languages`: every UTF-8 locale glibc supports (/usr/share/i18n/
//!   SUPPORTED), with the names glibc's own locale sources carry — the
//!   language in English and in itself ("Georgian" / "ქართული"), the region
//!   likewise, the ISO codes — and the spelling dictionary that goes with it
//!   (the table ewe Settings uses, `ewe-settings/src-tauri/src/language.rs`).
//!   The frontend prefers the webview's `Intl.DisplayNames`; these names are
//!   the fallback when the live ISO's WebKit has no data.
//! - `keyboard_db`: every xkb layout and variant with the languages it is
//!   for (evdev.xml's `<languageList>`, ISO 639-2/T codes = glibc's
//!   `lang_term`), so a language can suggest its keyboard.
//! - `ime_catalog`: the input methods ewe offers (`ewe-ime catalog`, the
//!   payload's own list — nothing to keep in step here).
//! - `try_layouts`: the chosen keyboards applied to the live session at once
//!   (Hyprland, the live user's own compositor), so the test field tests and
//!   the password is typed in the layout the greeter will use.
//!
//! Nothing here writes anything but the live session's keyboard.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde_json::{json, Value};
use tokio::process::Command;

const SUPPORTED: &str = "/usr/share/i18n/SUPPORTED";
const LOCALE_SRC: &str = "/usr/share/i18n/locales";
const EVDEV_XML: &str = "/usr/share/X11/xkb/rules/evdev.xml";

/// Every dictionary the official repositories install, with its package —
/// copied from ewe Settings (`language.rs` OFFICIAL, 2026-11-21); keep the two
/// in step. `en_US` comes with ewe itself (hunspell-en_us is in common.list).
const OFFICIAL: &[(&str, &str)] = &[
    ("de_DE", "hunspell-de"),
    ("de_AT", "hunspell-de"),
    ("de_CH", "hunspell-de"),
    ("de_BE", "hunspell-de"),
    ("de_LI", "hunspell-de"),
    ("de_LU", "hunspell-de"),
    ("el_GR", "hunspell-el"),
    ("en_US", "hunspell-en_us"),
    ("en_GB", "hunspell-en_gb"),
    ("en_AU", "hunspell-en_au"),
    ("en_CA", "hunspell-en_ca"),
    ("en_IE", "hunspell-en_gb"),
    ("en_NZ", "hunspell-en_gb"),
    ("en_ZA", "hunspell-en_gb"),
    ("en_IN", "hunspell-en_gb"),
    ("en_SG", "hunspell-en_gb"),
    ("en_HK", "hunspell-en_gb"),
    ("en_DK", "hunspell-en_gb"),
    ("en_NG", "hunspell-en_gb"),
    ("en_GH", "hunspell-en_gb"),
    ("en_JM", "hunspell-en_gb"),
    ("en_TT", "hunspell-en_gb"),
    ("en_ZW", "hunspell-en_gb"),
    ("en_BW", "hunspell-en_gb"),
    ("en_NA", "hunspell-en_gb"),
    ("en_BZ", "hunspell-en_gb"),
    ("en_BS", "hunspell-en_gb"),
    ("en_AG", "hunspell-en_gb"),
    ("es_ES", "hunspell-es_es"),
    ("es_MX", "hunspell-es_mx"),
    ("es_AR", "hunspell-es_ar"),
    ("es_BO", "hunspell-es_bo"),
    ("es_CL", "hunspell-es_cl"),
    ("es_CO", "hunspell-es_co"),
    ("es_CR", "hunspell-es_cr"),
    ("es_CU", "hunspell-es_cu"),
    ("es_DO", "hunspell-es_do"),
    ("es_EC", "hunspell-es_ec"),
    ("es_GT", "hunspell-es_gt"),
    ("es_HN", "hunspell-es_hn"),
    ("es_NI", "hunspell-es_ni"),
    ("es_PA", "hunspell-es_pa"),
    ("es_PE", "hunspell-es_pe"),
    ("es_PR", "hunspell-es_pr"),
    ("es_PY", "hunspell-es_py"),
    ("es_SV", "hunspell-es_sv"),
    ("es_UY", "hunspell-es_uy"),
    ("es_VE", "hunspell-es_ve"),
    ("fr_FR", "hunspell-fr-comprehensive"),
    ("fr_BE", "hunspell-fr-comprehensive"),
    ("fr_CA", "hunspell-fr-comprehensive"),
    ("fr_CH", "hunspell-fr-comprehensive"),
    ("fr_LU", "hunspell-fr-comprehensive"),
    ("he_IL", "hunspell-he"),
    ("hu_HU", "hunspell-hu"),
    ("it_IT", "hunspell-it"),
    ("it_CH", "hunspell-it"),
    ("nl_NL", "hunspell-nl"),
    ("nl_BE", "hunspell-nl"),
    ("nl_AW", "hunspell-nl"),
    ("pl_PL", "hunspell-pl"),
    ("ro_RO", "hunspell-ro"),
    ("ru_RU", "hunspell-ru"),
];

/// Languages whose dictionary only the AUR has: language (or locale) code,
/// dictionary name, AUR package — ewe Settings' AUR table. The installer
/// never builds from the AUR; it only says where the dictionary can come
/// from later (ewe Settings → Language, through Komble).
const AUR: &[(&str, &str, &str)] = &[
    ("af", "af_ZA", "hunspell-af"),
    ("ar", "ar", "hunspell-ar"),
    ("be", "be_BY", "hunspell-be"),
    ("bg", "bg_BG", "hunspell-bg"),
    ("bn", "bn_BD", "hunspell-bn"),
    ("ca", "ca_ES", "hunspell-ca"),
    ("cs", "cs_CZ", "hunspell-cs"),
    ("eo", "eo", "hunspell-eo"),
    ("et", "et_EE", "hunspell-et"),
    ("fa", "fa_IR", "hunspell-fa"),
    ("fo", "fo_FO", "hunspell-fo"),
    ("ga", "ga_IE", "hunspell-ga"),
    ("gl", "gl_ES", "hunspell-gl"),
    ("hi", "hi_IN", "hunspell-hi"),
    ("hr", "hr_HR", "hunspell-hr"),
    ("hy", "hy_AM", "hunspell-hy-reformed"),
    ("id", "id_ID", "hunspell-id"),
    ("is", "is_IS", "hunspell-is"),
    ("ka", "ka_GE", "hunspell-ka"),
    ("ko", "ko_KR", "hunspell-ko"),
    ("lb", "lb_LU", "hunspell-lb"),
    ("lt", "lt_LT", "hunspell-lt"),
    ("lv", "lv_LV", "hunspell-lv"),
    ("nb", "nb_NO", "hunspell-nb"),
    ("nn", "nn_NO", "hunspell-nn"),
    ("pt_BR", "pt_BR", "hunspell-pt-br"),
    ("pt_PT", "pt_PT", "hunspell-pt_pt"),
    ("pt", "pt_PT", "hunspell-pt_pt"),
    ("si", "si_LK", "hunspell-si"),
    ("sk", "sk_SK", "hunspell-sk"),
    ("sl", "sl_SI", "hunspell-sl"),
    ("sr", "sr", "hunspell-sr"),
    ("sv", "sv_SE", "hunspell-sv"),
    ("th", "th_TH", "hunspell-th"),
    ("tr", "tr_TR", "hunspell-tr"),
    ("uk", "uk_UA", "hunspell-uk"),
    ("ur", "ur_PK", "hunspell-ur"),
    ("vi", "vi_VN", "hunspell-vi"),
];

/// "de_DE.UTF-8@euro" → "de_DE"; "aa_ER" → "aa_ER"
fn locale_base(locale: &str) -> &str {
    let l = locale.split('@').next().unwrap_or(locale);
    l.split('.').next().unwrap_or(l)
}

/// The spelling dictionary for a locale: (name, Some(official package)) or
/// (name, None) + the AUR package. Mirrors ewe Settings' `dict_for_locale`
/// with nothing installed yet: the locale's own dictionary, else the first
/// official one of the language (en_PH → en_US, ru_UA → ru_RU).
pub(crate) fn dict_for(
    locale: &str,
) -> Option<(String, Option<&'static str>, Option<&'static str>)> {
    let base = locale_base(locale);
    let lang = base.split('_').next().unwrap_or(base);
    if lang.is_empty() || lang == "C" || lang == "POSIX" {
        return None;
    }
    let official = |d: &str| OFFICIAL.iter().find(|(c, _)| *c == d).map(|(_, p)| *p);
    if let Some(p) = official(base) {
        return Some((base.to_string(), Some(p), None));
    }
    if let Some((_, d, p)) = AUR.iter().find(|(l, _, _)| *l == base || *l == lang) {
        return Some(((*d).to_string(), None, Some(*p)));
    }
    let pref = format!("{lang}_{}", lang.to_uppercase());
    if let Some(p) = official(&pref) {
        return Some((pref, Some(p), None));
    }
    if lang == "en" {
        return Some(("en_US".into(), Some("hunspell-en_us"), None));
    }
    OFFICIAL
        .iter()
        .find(|(c, _)| c.split('_').next() == Some(lang))
        .map(|(c, p)| ((*c).to_string(), Some(*p), None))
}

/// SUPPORTED's UTF-8 rows: "en_US.UTF-8 UTF-8" → "en_US.UTF-8", "aa_ER
/// UTF-8" → "aa_ER" (the first column is the LANG value; the charset
/// column decides). C.UTF-8 is not a language.
pub(crate) fn supported_in(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let code = it.next()?;
            let charset = it.next().unwrap_or("");
            (charset == "UTF-8" || (charset.is_empty() && code.ends_with(".UTF-8")))
                .then(|| code.to_string())
        })
        .filter(|c| {
            !c.is_empty()
                && c.len() <= 64
                && !c.starts_with("C.")
                && c.chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || "_.@-".contains(ch))
        })
        .collect()
}

/// The locale source file for a SUPPORTED code: "sr_RS.UTF-8@latin" →
/// "sr_RS@latin", "ka_GE.UTF-8" → "ka_GE".
fn source_name(code: &str) -> String {
    let (rest, modifier) = match code.split_once('@') {
        Some((b, m)) => (b, Some(m)),
        None => (code, None),
    };
    let base = rest.split('.').next().unwrap_or(rest);
    match modifier {
        Some(m) => format!("{base}@{m}"),
        None => base.to_string(),
    }
}

/// glibc writes non-ASCII either as UTF-8 or as <U10E5> escapes; both read
/// back as text here.
fn decode_value(v: &str) -> String {
    let mut out = String::new();
    let mut rest = v;
    while let Some(i) = rest.find("<U") {
        out.push_str(&rest[..i]);
        let tail = &rest[i + 2..];
        match tail.find('>') {
            Some(j) if j > 0 && j <= 8 && tail[..j].chars().all(|c| c.is_ascii_hexdigit()) => {
                if let Some(ch) = u32::from_str_radix(&tail[..j], 16)
                    .ok()
                    .and_then(char::from_u32)
                {
                    out.push(ch);
                }
                rest = &tail[j + 1..];
            }
            _ => {
                out.push_str("<U");
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The `key "value"` lines of ONE category (LC_ADDRESS, LC_IDENTIFICATION)
/// of a locale source, following `copy "other"` up to three hops — many
/// locales copy a category from a sibling.
fn category_fields(dir: &Path, name: &str, cat: &str, depth: u8) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(dir.join(name)) else {
        return out;
    };
    let mut inside = false;
    let mut copy: Option<String> = None;
    for line in text.lines() {
        let t = line.trim();
        if !inside {
            inside = t == cat;
            continue;
        }
        if t == format!("END {cat}") {
            break;
        }
        let Some((key, rest)) = t.split_once(char::is_whitespace) else {
            continue;
        };
        let rest = rest.trim();
        let Some(val) = rest
            .strip_prefix('"')
            .and_then(|r| r.split_once('"'))
            .map(|(v, _)| v)
        else {
            continue;
        };
        if key == "copy" {
            copy = Some(val.to_string());
        } else {
            out.insert(key.to_string(), decode_value(val));
        }
    }
    if let Some(other) = copy {
        if depth < 3 && other != name {
            for (k, v) in category_fields(dir, &other, cat, depth + 1) {
                out.entry(k).or_insert(v);
            }
        }
    }
    out
}

/// One row per UTF-8 locale. `lang`/`country` come from the code itself
/// when the source file is missing, so a trimmed live system still lists
/// every locale (with its code as the name).
pub(crate) fn languages_in(supported: &str, src: &Path) -> Vec<Value> {
    let mut out = Vec::new();
    for code in supported_in(supported) {
        let name = source_name(&code);
        let id = category_fields(src, &name, "LC_IDENTIFICATION", 0);
        let addr = category_fields(src, &name, "LC_ADDRESS", 0);
        let base = locale_base(&code).to_string();
        let (code_lang, code_country) = match base.split_once('_') {
            Some((l, c)) => (l.to_string(), c.to_string()),
            None => (base.clone(), String::new()),
        };
        let field = |m: &BTreeMap<String, String>, k: &str| m.get(k).cloned().unwrap_or_default();
        let modifier = code.split_once('@').map(|(_, m)| m).unwrap_or("");
        let (dict, dict_aur) = match dict_for(&code) {
            Some((d, Some(p), _)) => (json!({"name": d, "package": p}), Value::Null),
            Some((d, None, Some(p))) => (Value::Null, json!({"name": d, "package": p})),
            _ => (Value::Null, Value::Null),
        };
        out.push(json!({
            "locale": code,
            // the code's own language part: what groups the list ("pt",
            // "ckb", "sr") — lang_ab is two letters only and absent for many
            "lang": code_lang,
            "term": field(&addr, "lang_term"),
            "country": if code_country.is_empty() { field(&addr, "country_ab2") } else { code_country },
            "modifier": modifier,
            "language": field(&id, "language"),
            "territory": field(&id, "territory"),
            "lang_name": field(&addr, "lang_name"),
            "country_name": field(&addr, "country_name"),
            "dict": dict,
            "dict_aur": dict_aur,
        }));
    }
    out
}

#[tauri::command]
pub async fn languages() -> Result<Value, String> {
    let text = std::fs::read_to_string(SUPPORTED).unwrap_or_default();
    Ok(json!(languages_in(&text, Path::new(LOCALE_SRC))))
}

// ── xkb: layouts, variants, the languages they are for ─────────────────────

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// The text of the first `<tag>…</tag>` in `s`.
fn tag_text<'a>(s: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let i = s.find(&open)? + open.len();
    let j = s[i..].find(&close)? + i;
    Some(s[i..j].trim())
}

/// Every `<tag>…</tag>` text in `s`.
fn tag_texts<'a>(s: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find(&open) {
        let after = &rest[i + open.len()..];
        let Some(j) = after.find(&close) else { break };
        out.push(after[..j].trim());
        rest = &after[j + close.len()..];
    }
    out
}

/// The blocks between `<tag>` and `</tag>` (not nested in themselves).
fn blocks<'a>(s: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find(&open) {
        let after = &rest[i + open.len()..];
        let Some(j) = after.find(&close) else { break };
        out.push(&after[..j]);
        rest = &after[j + close.len()..];
    }
    out
}

/// evdev.xml → [{c, v, n, langs}] — one row per layout (v = "") and per
/// variant, in the file's order. `langs` are ISO 639-2/T codes; a variant
/// without a list of its own inherits the layout's.
pub(crate) fn keyboard_db_in(xml: &str) -> Vec<Value> {
    let Some(start) = xml.find("<layoutList>") else {
        return Vec::new();
    };
    let end = xml[start..]
        .find("</layoutList>")
        .map(|e| start + e)
        .unwrap_or(xml.len());
    let list = &xml[start..end];
    let mut out = Vec::new();
    for layout in blocks(list, "layout") {
        let (head, variants) = match layout.find("<variantList>") {
            Some(i) => (&layout[..i], &layout[i..]),
            None => (layout, ""),
        };
        let Some(code) = tag_text(head, "name") else {
            continue;
        };
        let name = unescape_xml(tag_text(head, "description").unwrap_or(code));
        let langs: Vec<String> = tag_texts(head, "iso639Id")
            .into_iter()
            .map(String::from)
            .collect();
        out.push(json!({"c": code, "v": "", "n": name, "langs": langs}));
        for var in blocks(variants, "variant") {
            let Some(vc) = tag_text(var, "name") else {
                continue;
            };
            let vn = unescape_xml(tag_text(var, "description").unwrap_or(vc));
            let own: Vec<String> = tag_texts(var, "iso639Id")
                .into_iter()
                .map(String::from)
                .collect();
            out.push(json!({"c": code, "v": vc, "n": vn,
                            "langs": if own.is_empty() { langs.clone() } else { own }}));
        }
    }
    out
}

#[tauri::command]
pub async fn keyboard_db() -> Result<Value, String> {
    let xml = std::fs::read_to_string(EVDEV_XML).unwrap_or_default();
    Ok(json!(keyboard_db_in(&xml)))
}

// ── typing in other languages: the payload's own catalogue ─────────────────

fn ewe_ime() -> Option<PathBuf> {
    ["/usr/bin/ewe-ime", "/usr/share/ewe/bin/ewe-ime"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

/// `ewe-ime catalog` → [{id, language, native, method, packages}]; an ISO
/// with an ewe older than 0.46 has none, and the Language screen then
/// offers no input method (nothing else changes).
#[tauri::command]
pub async fn ime_catalog() -> Result<Value, String> {
    let Some(bin) = ewe_ime() else {
        return Ok(json!([]));
    };
    let out = Command::new(bin)
        .arg("catalog")
        .stdin(Stdio::null())
        .output()
        .await;
    let v: Value = out
        .ok()
        .and_then(|o| serde_json::from_slice(&o.stdout).ok())
        .unwrap_or(Value::Null);
    let rows: Vec<Value> = v["catalog"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r["id"].as_str().is_some_and(ime_id_ok))
        .map(|r| {
            json!({
                "id": r["id"], "language": r["language"], "native": r["native"],
                "method": r["method"], "package": r["package"],
                "packages": r["packages"],
            })
        })
        .collect();
    Ok(json!(rows))
}

/// ewe-ime's own id shape (ID_RE): what the helper accepts too.
pub(crate) fn ime_id_ok(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))
}

// ── the chosen keyboards, live ─────────────────────────────────────────────

/// xkb layout / variant lists as Hyprland takes them: "us,ge" / ",". Lower
/// case, digits and _ for layouts; variants may be empty or carry '-'.
pub(crate) fn layouts_ok(layouts: &str, variants: &str) -> bool {
    let ls: Vec<&str> = layouts.split(',').collect();
    !layouts.is_empty()
        && ls.len() <= 4
        && ls.iter().all(|l| {
            !l.is_empty()
                && l.len() <= 32
                && l.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
        && (variants.is_empty() || variants.split(',').count() == ls.len())
        && variants.len() <= 160
        && variants
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-,".contains(c))
}

/// Apply the layouts to the live session (Hyprland's own `hl.config`). A
/// failure is not an error for the UI: the choice still goes to the new
/// system; only the try-it field would type the old way.
#[tauri::command]
pub async fn try_layouts(layouts: String, variants: String) -> Result<bool, String> {
    if !layouts_ok(&layouts, &variants) {
        return Err("not a keyboard layout".into());
    }
    let stmt = format!(
        "hl.config({{ input = {{ kb_layout = \"{layouts}\", kb_variant = \"{variants}\" }} }})"
    );
    let out = Command::new("hyprctl")
        .args(["eval", &stmt])
        .stdin(Stdio::null())
        .output()
        .await;
    Ok(matches!(out, Ok(o) if String::from_utf8_lossy(&o.stdout).trim_start().starts_with("ok")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("ewe-installer-lang-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn supported_utf8_rows_only() {
        let s = "en_US.UTF-8 UTF-8\nen_US ISO-8859-1\naa_ER UTF-8\nC.UTF-8 UTF-8\n\
                 sr_RS.UTF-8@latin UTF-8\nde_DE@euro ISO-8859-15\nbad;rm UTF-8\n";
        assert_eq!(
            supported_in(s),
            vec!["en_US.UTF-8", "aa_ER", "sr_RS.UTF-8@latin"]
        );
    }

    #[test]
    fn names_from_the_locale_sources_follow_copy_and_escapes() {
        let d = tmp("src");
        std::fs::write(
            d.join("ka_GE"),
            "LC_IDENTIFICATION\nlanguage \"Georgian\"\nterritory \"Georgia\"\nEND LC_IDENTIFICATION\n\
             LC_ADDRESS\ncountry_name \"საქართველო\"\ncountry_ab2 \"GE\"\nlang_name    \"ქართული\"\n\
             lang_term \"kat\"\nEND LC_ADDRESS\n",
        )
        .unwrap();
        std::fs::write(
            d.join("xx_YY"),
            "LC_IDENTIFICATION\nlanguage \"Example\"\nEND LC_IDENTIFICATION\n\
             LC_ADDRESS\ncopy \"xx_ZZ\"\nEND LC_ADDRESS\n",
        )
        .unwrap();
        std::fs::write(
            d.join("xx_ZZ"),
            "LC_ADDRESS\nlang_name \"<U10E5><U10D0>x\"\nlang_term \"xxx\"\nEND LC_ADDRESS\n",
        )
        .unwrap();
        let rows = languages_in(
            "ka_GE.UTF-8 UTF-8\nxx_YY.UTF-8 UTF-8\nzz_QQ.UTF-8 UTF-8\n",
            &d,
        );
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["lang"], "ka");
        assert_eq!(rows[0]["country"], "GE");
        assert_eq!(rows[0]["lang_name"], "ქართული");
        assert_eq!(rows[0]["language"], "Georgian");
        assert_eq!(rows[0]["term"], "kat");
        // no official Georgian dictionary: the AUR one is named, not offered
        assert!(rows[0]["dict"].is_null());
        assert_eq!(rows[0]["dict_aur"]["package"], "hunspell-ka");
        assert_eq!(rows[1]["lang_name"], "ქაx");
        assert_eq!(rows[1]["term"], "xxx");
        // no source file at all: the code is still listed
        assert_eq!(rows[2]["lang"], "zz");
        assert_eq!(rows[2]["country"], "QQ");
        assert_eq!(rows[2]["language"], "");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn dictionaries_for_locales() {
        assert_eq!(dict_for("de_AT.UTF-8").unwrap().0, "de_AT");
        assert_eq!(dict_for("de_AT.UTF-8").unwrap().1, Some("hunspell-de"));
        assert_eq!(dict_for("ja_JP.UTF-8"), None);
        let ka = dict_for("ka_GE.UTF-8").unwrap();
        assert_eq!(
            (ka.0.as_str(), ka.1, ka.2),
            ("ka_GE", None, Some("hunspell-ka"))
        );
        assert_eq!(dict_for("en_PH.UTF-8").unwrap().0, "en_US");
        assert_eq!(dict_for("ru_UA.UTF-8").unwrap().0, "ru_RU");
        assert_eq!(dict_for("pt_BR.UTF-8").unwrap().2, Some("hunspell-pt-br"));
        assert_eq!(dict_for("C.UTF-8"), None);
    }

    #[test]
    fn source_names() {
        assert_eq!(source_name("sr_RS.UTF-8@latin"), "sr_RS@latin");
        assert_eq!(source_name("ka_GE.UTF-8"), "ka_GE");
        assert_eq!(source_name("aa_ER"), "aa_ER");
    }

    #[test]
    fn xkb_layouts_variants_and_languages() {
        let xml = r#"<xkbConfigRegistry><modelList><model><configItem><name>pc105</name></configItem></model></modelList>
<layoutList>
<layout><configItem><name>ge</name><shortDescription>ka</shortDescription><description>Georgian</description>
<languageList><iso639Id>kat</iso639Id></languageList></configItem>
<variantList><variant><configItem><name>qwerty</name><description>Georgian (QWERTY)</description></configItem></variant>
<variant><configItem><name>ru</name><description>Russian (Georgia)</description><languageList><iso639Id>rus</iso639Id></languageList></configItem></variant></variantList>
</layout>
<layout><configItem><name>us</name><description>English (US)</description><languageList><iso639Id>eng</iso639Id></languageList></configItem></layout>
<layout><configItem><name>tm</name><description>Turkmen &amp; more</description></configItem></layout>
</layoutList><optionList/></xkbConfigRegistry>"#;
        let db = keyboard_db_in(xml);
        let rows: Vec<(String, String, String)> = db
            .iter()
            .map(|r| {
                (
                    r["c"].as_str().unwrap().into(),
                    r["v"].as_str().unwrap().into(),
                    r["n"].as_str().unwrap().into(),
                )
            })
            .collect();
        assert_eq!(rows[0], ("ge".into(), "".into(), "Georgian".into()));
        assert_eq!(rows[1].1, "qwerty");
        assert_eq!(db[1]["langs"], json!(["kat"])); // inherits the layout's
        assert_eq!(db[2]["langs"], json!(["rus"])); // its own
        assert_eq!(rows[3].0, "us");
        assert_eq!(rows[4].2, "Turkmen & more");
        assert_eq!(db.len(), 5);
    }

    #[test]
    fn the_real_registry_when_present() {
        let Ok(xml) = std::fs::read_to_string(EVDEV_XML) else {
            return;
        };
        let db = keyboard_db_in(&xml);
        assert!(db.len() > 300, "layouts + variants: {}", db.len());
        let ge = db.iter().find(|r| r["c"] == "ge" && r["v"] == "").unwrap();
        assert!(ge["langs"].as_array().unwrap().contains(&json!("kat")));
        let jp = db.iter().find(|r| r["c"] == "jp" && r["v"] == "").unwrap();
        assert!(jp["langs"].as_array().unwrap().contains(&json!("jpn")));
    }

    #[test]
    fn the_real_locales_when_present() {
        let Ok(text) = std::fs::read_to_string(SUPPORTED) else {
            return;
        };
        let rows = languages_in(&text, Path::new(LOCALE_SRC));
        let ka = rows.iter().find(|r| r["locale"] == "ka_GE.UTF-8").unwrap();
        assert_eq!(ka["lang_name"], "ქართული");
        assert_eq!(ka["term"], "kat");
        let ja = rows.iter().find(|r| r["locale"] == "ja_JP.UTF-8").unwrap();
        assert_eq!(ja["lang_name"], "日本語");
        assert!(rows.iter().all(|r| !r["lang"].as_str().unwrap().is_empty()));
    }

    #[test]
    fn layout_lists() {
        assert!(layouts_ok("us", ""));
        assert!(layouts_ok("us,ge", ""));
        assert!(layouts_ok("us,ge", ",qwerty"));
        assert!(layouts_ok("de", "nodeadkeys"));
        assert!(layouts_ok("us,in", ",hin-wx"));
        assert!(!layouts_ok("", ""));
        assert!(!layouts_ok("us,ge", "x"));
        assert!(!layouts_ok("us\"", ""));
        assert!(!layouts_ok("us", "a\" }) os.execute(\"x"));
        assert!(!layouts_ok("a,b,c,d,e", ""));
    }

    /// The browser fixture (src/lib/mock-lang.js) is cut from this machine's
    /// real data: `EWE_LANG_DUMP=<dir> cargo test dump -- --ignored`.
    #[test]
    #[ignore = "writes the dev fixture's source data"]
    fn dump() {
        let Some(dir) = std::env::var_os("EWE_LANG_DUMP") else {
            return;
        };
        let dir = PathBuf::from(dir);
        let text = std::fs::read_to_string(SUPPORTED).unwrap_or_default();
        let rows = languages_in(&text, Path::new(LOCALE_SRC));
        std::fs::write(
            dir.join("languages.json"),
            serde_json::to_string(&rows).unwrap(),
        )
        .unwrap();
        let xml = std::fs::read_to_string(EVDEV_XML).unwrap_or_default();
        let db = keyboard_db_in(&xml);
        std::fs::write(dir.join("kbdb.json"), serde_json::to_string(&db).unwrap()).unwrap();
    }

    #[test]
    fn ime_ids() {
        assert!(ime_id_ok("mozc"));
        assert!(ime_id_ok("m17n_pa_inscript2-guru"));
        assert!(!ime_id_ok("mozc;rm"));
        assert!(!ime_id_ok(""));
    }
}
