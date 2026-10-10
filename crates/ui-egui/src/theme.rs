//! Design tokens and fonts.
//!
//! Colours are our own, tuned to read like a modern desktop spreadsheet. Fonts come from the
//! operating system at runtime (nothing bundled): the UI uses the platform UI font, cells use the
//! best available match for the workbook font (Calibri/Carlito/Aptos/Arial…) with egui's built-in
//! fonts as the last resort.

use std::sync::Arc;

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, Visuals};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub dark: bool,
    pub window: Color32,
    pub ribbon: Color32,
    pub ribbon_border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_disabled: Color32,
    pub accent: Color32,
    pub accent_dark: Color32,
    pub accent_soft: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub separator: Color32,
    pub grid_bg: Color32,
    pub gridline: Color32,
    pub header_bg: Color32,
    pub header_text: Color32,
    pub header_line: Color32,
    pub header_sel_bg: Color32,
    pub header_sel_text: Color32,
    pub header_all_bg: Color32,
    pub sel_fill: Color32,
    pub sel_border: Color32,
    pub input_bg: Color32,
    pub input_border: Color32,
    pub tab_bar: Color32,
    pub tab_active: Color32,
    pub status_bar: Color32,
    pub cell_text: Color32,
    pub menu_bg: Color32,
    pub shadow: Color32,
    pub danger: Color32,
}

impl Tokens {
    pub fn light() -> Tokens {
        Tokens {
            dark: false,
            window: Color32::from_rgb(0xF3, 0xF3, 0xF3),
            ribbon: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            ribbon_border: Color32::from_rgb(0xE3, 0xE3, 0xE3),
            text: Color32::from_rgb(0x24, 0x24, 0x24),
            text_dim: Color32::from_rgb(0x61, 0x61, 0x61),
            text_disabled: Color32::from_rgb(0xB0, 0xB0, 0xB0),
            accent: Color32::from_rgb(0x10, 0x7C, 0x41),
            accent_dark: Color32::from_rgb(0x0B, 0x5E, 0x31),
            accent_soft: Color32::from_rgb(0xDF, 0xF1, 0xE6),
            hover: Color32::from_rgb(0xEB, 0xEB, 0xEB),
            pressed: Color32::from_rgb(0xDD, 0xDD, 0xDD),
            separator: Color32::from_rgb(0xE0, 0xE0, 0xE0),
            grid_bg: Color32::WHITE,
            gridline: Color32::from_rgb(0xE1, 0xE1, 0xE1),
            header_bg: Color32::from_rgb(0xF8, 0xF8, 0xF8),
            header_text: Color32::from_rgb(0x42, 0x42, 0x42),
            header_line: Color32::from_rgb(0xD6, 0xD6, 0xD6),
            header_sel_bg: Color32::from_rgb(0xE1, 0xE1, 0xE1),
            header_sel_text: Color32::from_rgb(0x10, 0x7C, 0x41),
            header_all_bg: Color32::from_rgb(0xC8, 0xE6, 0xD3),
            sel_fill: Color32::from_rgba_unmultiplied(0x10, 0x7C, 0x41, 0x1C),
            sel_border: Color32::from_rgb(0x10, 0x7C, 0x41),
            input_bg: Color32::WHITE,
            input_border: Color32::from_rgb(0xD1, 0xD1, 0xD1),
            tab_bar: Color32::from_rgb(0xF3, 0xF3, 0xF3),
            tab_active: Color32::WHITE,
            status_bar: Color32::from_rgb(0xF3, 0xF3, 0xF3),
            cell_text: Color32::BLACK,
            menu_bg: Color32::WHITE,
            shadow: Color32::from_black_alpha(40),
            danger: Color32::from_rgb(0xC4, 0x2B, 0x1C),
        }
    }

    pub fn dark() -> Tokens {
        Tokens {
            dark: true,
            window: Color32::from_rgb(0x29, 0x29, 0x29),
            ribbon: Color32::from_rgb(0x33, 0x33, 0x33),
            ribbon_border: Color32::from_rgb(0x40, 0x40, 0x40),
            text: Color32::from_rgb(0xEE, 0xEE, 0xEE),
            text_dim: Color32::from_rgb(0xB0, 0xB0, 0xB0),
            text_disabled: Color32::from_rgb(0x70, 0x70, 0x70),
            accent: Color32::from_rgb(0x5C, 0xC5, 0x87),
            accent_dark: Color32::from_rgb(0x3F, 0xA8, 0x6C),
            accent_soft: Color32::from_rgb(0x23, 0x45, 0x31),
            hover: Color32::from_rgb(0x42, 0x42, 0x42),
            pressed: Color32::from_rgb(0x4D, 0x4D, 0x4D),
            separator: Color32::from_rgb(0x48, 0x48, 0x48),
            grid_bg: Color32::WHITE,
            gridline: Color32::from_rgb(0xE1, 0xE1, 0xE1),
            header_bg: Color32::from_rgb(0x30, 0x30, 0x30),
            header_text: Color32::from_rgb(0xCC, 0xCC, 0xCC),
            header_line: Color32::from_rgb(0x45, 0x45, 0x45),
            header_sel_bg: Color32::from_rgb(0x45, 0x45, 0x45),
            header_sel_text: Color32::from_rgb(0x5C, 0xC5, 0x87),
            header_all_bg: Color32::from_rgb(0x2E, 0x55, 0x3E),
            sel_fill: Color32::from_rgba_unmultiplied(0x10, 0x7C, 0x41, 0x24),
            sel_border: Color32::from_rgb(0x10, 0x7C, 0x41),
            input_bg: Color32::from_rgb(0x1F, 0x1F, 0x1F),
            input_border: Color32::from_rgb(0x50, 0x50, 0x50),
            tab_bar: Color32::from_rgb(0x29, 0x29, 0x29),
            tab_active: Color32::from_rgb(0x3A, 0x3A, 0x3A),
            status_bar: Color32::from_rgb(0x29, 0x29, 0x29),
            cell_text: Color32::BLACK,
            menu_bg: Color32::from_rgb(0x30, 0x30, 0x30),
            shadow: Color32::from_black_alpha(90),
            danger: Color32::from_rgb(0xF1, 0x70, 0x5F),
        }
    }

    pub fn get(ctx: &egui::Context) -> Tokens {
        if ctx.global_style().visuals.dark_mode { Tokens::dark() } else { Tokens::light() }
    }
}

pub const UI: &str = "ui";
pub const UI_BOLD: &str = "ui-bold";
pub const CELL: &str = "cell";
pub const CELL_BOLD: &str = "cell-bold";
pub const CELL_ITALIC: &str = "cell-italic";
pub const CELL_BOLD_ITALIC: &str = "cell-bold-italic";
pub const SERIF: &str = "serif";
pub const MONO: &str = "mono";

pub fn ui_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(UI.into()))
}
pub fn ui_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(UI_BOLD.into()))
}

/// The font family to use for a cell font name and weight.
pub fn cell_family(name: &str, bold: bool, italic: bool) -> FontFamily {
    let lower = name.to_ascii_lowercase();
    if lower.contains("courier") || lower.contains("mono") || lower.contains("consolas") {
        return FontFamily::Name(MONO.into());
    }
    if lower.contains("times")
        || lower.contains("georgia")
        || lower.contains("cambria")
        || lower.contains("serif") && !lower.contains("sans")
        || lower.contains("garamond")
    {
        return FontFamily::Name(SERIF.into());
    }
    FontFamily::Name(
        match (bold, italic) {
            (true, true) => CELL_BOLD_ITALIC,
            (true, false) => CELL_BOLD,
            (false, true) => CELL_ITALIC,
            (false, false) => CELL,
        }
        .into(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn try_load(paths: &[(&str, u32)]) -> Option<FontData> {
    for (p, index) in paths {
        if let Ok(bytes) = std::fs::read(p) {
            let mut fd = FontData::from_owned(bytes);
            fd.index = *index;
            return Some(fd);
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn try_load(_paths: &[(&str, u32)]) -> Option<FontData> {
    None
}

fn home_fonts(file: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/Library/Fonts/{file}")
}

/// Candidate font files per role, in preference order.
fn candidates(role: &str) -> Vec<(String, u32)> {
    let mac_sys = "/System/Library/Fonts";
    let mac_sup = "/System/Library/Fonts/Supplemental";
    let win = std::env::var("WINDIR").map(|w| format!("{w}\\Fonts")).unwrap_or_else(|_| "C:\\Windows\\Fonts".into());
    let lin = ["/usr/share/fonts/truetype", "/usr/share/fonts", "/usr/local/share/fonts", "/usr/share/fonts/TTF"];
    let mut v: Vec<(String, u32)> = Vec::new();
    let mut add = |s: String, i: u32| v.push((s, i));
    match role {
        "ui" => {
            add(format!("{mac_sys}/SFNS.ttf"), 0);
            add(format!("{mac_sys}/HelveticaNeue.ttc"), 0);
            add(format!("{win}\\segoeui.ttf"), 0);
            for d in lin {
                add(format!("{d}/cantarell/Cantarell-Regular.otf"), 0);
                add(format!("{d}/noto/NotoSans-Regular.ttf"), 0);
                add(format!("{d}/dejavu/DejaVuSans.ttf"), 0);
                add(format!("{d}/liberation/LiberationSans-Regular.ttf"), 0);
            }
        }
        "ui-bold" => {
            add(format!("{mac_sys}/HelveticaNeue.ttc"), 1);
            add(format!("{win}\\segoeuib.ttf"), 0);
            for d in lin {
                add(format!("{d}/noto/NotoSans-Bold.ttf"), 0);
                add(format!("{d}/dejavu/DejaVuSans-Bold.ttf"), 0);
                add(format!("{d}/liberation/LiberationSans-Bold.ttf"), 0);
            }
        }
        "cell" | "cell-bold" | "cell-italic" | "cell-bold-italic" => {
            let (carlito, calibri_win, calibri_mac, arial_mac, arial_win, lib, dejavu) = match role {
                "cell" => {
                    ("Carlito-Regular.ttf", "calibri.ttf", "Calibri.ttf", "Arial.ttf", "arial.ttf", "LiberationSans-Regular.ttf", "DejaVuSans.ttf")
                }
                "cell-bold" => (
                    "Carlito-Bold.ttf",
                    "calibrib.ttf",
                    "Calibri Bold.ttf",
                    "Arial Bold.ttf",
                    "arialbd.ttf",
                    "LiberationSans-Bold.ttf",
                    "DejaVuSans-Bold.ttf",
                ),
                "cell-italic" => (
                    "Carlito-Italic.ttf",
                    "calibrii.ttf",
                    "Calibri Italic.ttf",
                    "Arial Italic.ttf",
                    "ariali.ttf",
                    "LiberationSans-Italic.ttf",
                    "DejaVuSans-Oblique.ttf",
                ),
                _ => (
                    "Carlito-BoldItalic.ttf",
                    "calibriz.ttf",
                    "Calibri Bold Italic.ttf",
                    "Arial Bold Italic.ttf",
                    "arialbi.ttf",
                    "LiberationSans-BoldItalic.ttf",
                    "DejaVuSans-BoldOblique.ttf",
                ),
            };
            // Calibri if the user has it installed system-wide (never from an Office bundle).
            add(format!("/Library/Fonts/{calibri_mac}"), 0);
            add(home_fonts(calibri_mac), 0);
            add(format!("{win}\\{calibri_win}"), 0);
            for d in lin {
                add(format!("{d}/crosextra/{carlito}"), 0);
                add(format!("{d}/carlito/{carlito}"), 0);
            }
            add(home_fonts(carlito), 0);
            add(format!("{mac_sup}/{arial_mac}"), 0);
            add(format!("{win}\\{arial_win}"), 0);
            for d in lin {
                add(format!("{d}/liberation/{lib}"), 0);
                add(format!("{d}/dejavu/{dejavu}"), 0);
            }
        }
        "serif" => {
            add(format!("{mac_sup}/Times New Roman.ttf"), 0);
            add(format!("{win}\\times.ttf"), 0);
            for d in lin {
                add(format!("{d}/liberation/LiberationSerif-Regular.ttf"), 0);
                add(format!("{d}/dejavu/DejaVuSerif.ttf"), 0);
            }
        }
        "mono" => {
            add(format!("{mac_sys}/SFNSMono.ttf"), 0);
            add(format!("{mac_sys}/Menlo.ttc"), 0);
            add(format!("{win}\\consola.ttf"), 0);
            for d in lin {
                add(format!("{d}/dejavu/DejaVuSansMono.ttf"), 0);
            }
        }
        _ => {}
    }
    v
}

/// A font from the optional craft-fonts build input (empty unless built with `CRAFT_FONTS_DIR`).
pub struct CraftFont {
    pub family: &'static str,
    pub style: &'static str,
    /// ISO 15924 scripts the font is for, e.g. `"Jpan"`.
    pub scripts: &'static [&'static str],
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));

/// Whether `c` belongs to a script that needs a CJK font (Han, kana, Hangul, full-width forms).
pub fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x11FF | 0x2E80..=0x303F | 0x3040..=0x30FF | 0x3100..=0x318F | 0x31A0..=0x31FF
        | 0x3200..=0x4DBF | 0x4E00..=0x9FFF | 0xA960..=0xA97F | 0xAC00..=0xD7FF | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F | 0xFF00..=0xFFEF | 0x20000..=0x3FFFF)
}

/// Whether `s` contains a character that needs a CJK font.
pub fn has_cjk(s: &str) -> bool {
    s.chars().any(is_cjk)
}

/// Whether the interface language itself is written with CJK characters.
pub fn is_cjk_language(tag: &str) -> bool {
    ["ja", "ko", "zh"].iter().any(|p| tag == *p || tag.starts_with(&format!("{p}-")))
}

/// File name, below `./fonts/` of the page, of the CJK font the web build loads on demand for
/// the interface language `tag` (ja-JP → `cjk-ja.ttf`, ko-KR → `cjk-ko.ttf`, zh-CN → `cjk-zh-CN.ttf`,
/// zh-TW → `cjk-zh-TW.ttf`); any other language that meets CJK text uses `cjk-ja.ttf`. Fonts are
/// never part of this repository: a deployment that does not ship the file keeps CJK glyphs
/// missing (shown as boxes) and the host reports the failed download.
pub fn web_cjk_font(tag: &str) -> &'static str {
    if tag.starts_with("ko") {
        "cjk-ko.ttf"
    } else if tag == "zh-TW" || tag == "zh-HK" {
        "cjk-zh-TW.ttf"
    } else if tag.starts_with("zh") {
        "cjk-zh-CN.ttf"
    } else {
        "cjk-ja.ttf"
    }
}

/// The split Noto Sans CJK files in the order a language prefers them: its own first (ko → kr,
/// zh-TW/zh-HK → tc, zh-CN and other zh → sc, anything else jp), then the others.
#[cfg(not(target_arch = "wasm32"))]
fn noto_otf_order(tag: &str) -> [&'static str; 4] {
    let (jp, sc, kr, tc) = ("NotoSansCJKjp-Regular.otf", "NotoSansCJKsc-Regular.otf", "NotoSansCJKkr-Regular.otf", "NotoSansCJKtc-Regular.otf");
    if tag.starts_with("ko") {
        [kr, jp, sc, tc]
    } else if tag == "zh-TW" || tag == "zh-HK" {
        [tc, sc, jp, kr]
    } else if tag.starts_with("zh") {
        [sc, tc, jp, kr]
    } else {
        [jp, sc, kr, tc]
    }
}

/// System CJK fonts in preference order for the interface language `tag`: Han glyph shapes differ
/// between Japanese, Korean and Chinese, so the font of the user's language goes first.
#[cfg(not(target_arch = "wasm32"))]
fn cjk_candidates(tag: &str) -> Vec<(String, u32)> {
    let mac = "/System/Library/Fonts";
    let win = std::env::var("WINDIR").map(|w| format!("{w}\\Fonts")).unwrap_or_else(|_| "C:\\Windows\\Fonts".into());
    let lin = [
        "/usr/share/fonts/noto-cjk",
        "/usr/share/fonts/opentype/noto",
        "/usr/share/fonts/google-noto-cjk",
        "/usr/share/fonts/google-noto-sans-cjk-fonts",
        "/usr/share/fonts/truetype/noto",
        "/usr/share/fonts/noto",
        "/usr/share/fonts/OTF",
        "/usr/share/fonts/TTF",
    ];
    // Index of the language inside Noto Sans CJK collections: JP, KR, SC, TC, HK.
    let noto = if tag.starts_with("ko") {
        1
    } else if tag == "zh-TW" || tag == "zh-HK" {
        3
    } else if tag.starts_with("zh") {
        2
    } else {
        0
    };
    let mut v: Vec<(String, u32)> = Vec::new();
    let mut native: Vec<(String, u32)> = Vec::new();
    if tag.starts_with("ko") {
        native.push((format!("{mac}/AppleSDGothicNeo.ttc"), 0));
        native.push((format!("{win}\\malgun.ttf"), 0));
    } else if tag.starts_with("zh") {
        native.push((format!("{mac}/PingFang.ttc"), 0));
        native.push((format!("{win}\\{}", if tag == "zh-TW" { "msjh.ttc" } else { "msyh.ttc" }), 0));
    } else {
        native.push((format!("{mac}/ヒラギノ角ゴシック W3.ttc"), 0));
        native.push((format!("{win}\\meiryo.ttc"), 0));
        native.push((format!("{win}\\YuGothR.ttc"), 0));
    }
    v.extend(native);
    for d in lin {
        v.push((format!("{d}/NotoSansCJK-Regular.ttc"), noto));
        for file in noto_otf_order(tag) {
            v.push((format!("{d}/{file}"), 0));
        }
        v.push((format!("{d}/NotoSansCJK-VF.otf.ttc"), noto));
    }
    for p in [
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/droid/DroidSansFallbackFull.ttf",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
    ] {
        v.push((p.to_string(), 0));
    }
    v.push((format!("{mac}/PingFang.ttc"), 0));
    v.push((format!("{mac}/Hiragino Sans GB.ttc"), 0));
    v.push((format!("{win}\\msyh.ttc"), 0));
    v.push((format!("{win}\\meiryo.ttc"), 0));
    v.push((format!("{win}\\malgun.ttf"), 0));
    v.push((format!("{win}\\simsun.ttc"), 0));
    v
}

#[cfg(target_arch = "wasm32")]
fn cjk_candidates(_tag: &str) -> Vec<(String, u32)> {
    Vec::new()
}

/// What the font set has to cover beyond Latin text.
#[derive(Clone, Debug, Default)]
pub struct FontNeeds {
    /// Interface language tag (`ja-JP`); picks the CJK face whose Han glyph shapes match.
    pub language: String,
    /// The interface or a workbook uses CJK characters: add the CJK fallback chain.
    pub cjk: bool,
    /// Font files the host delivered (web build: `fonts/<name>` downloaded on demand).
    pub downloaded: Vec<(String, Arc<FontData>)>,
}

/// What decides whether the installed fonts must be rebuilt: the interface language only counts
/// while the CJK chain is on (it orders the faces), and the number of downloaded fonts.
pub type FontKey = (Option<String>, bool, usize);

impl FontNeeds {
    pub fn key(&self) -> FontKey {
        (self.cjk.then(|| self.language.clone()), self.cjk, self.downloaded.len())
    }
}

/// Builds the font set from system fonts (with egui's defaults as fallback).
///
/// With `needs.cjk` every family gets a fallback chain for CJK glyphs after the app's own fonts:
/// the craft-fonts faces built into the binary (see `CRAFT_FONTS`), the fonts the host
/// downloaded, and a system font (Noto Sans CJK, PingFang or Hiragino, Microsoft YaHei or Meiryo,
/// Malgun Gothic).
pub fn font_definitions(needs: &FontNeeds) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let base_prop: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let base_mono: Vec<String> = fonts.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    let mut cjk_chain: Vec<String> = Vec::new();
    if needs.cjk {
        let script = if needs.language.starts_with("ko") {
            "Kore"
        } else if needs.language == "zh-TW" || needs.language == "zh-HK" {
            "Hant"
        } else if needs.language.starts_with("zh") {
            "Hans"
        } else {
            "Jpan"
        };
        let add_craft = |fonts: &mut FontDefinitions, chain: &mut Vec<String>, f: &CraftFont| {
            let key = format!("craft-{}-{}", f.family, f.style);
            fonts.font_data.insert(key.clone(), Arc::new(FontData::from_static(f.bytes)));
            chain.push(key);
        };
        // Han glyph shapes differ per language: the font of the interface language goes first
        // (craft-fonts built for the script, then downloads, then a system font) and the
        // Japanese craft fonts only back it up, unless the interface itself is Japanese.
        for f in CRAFT_FONTS.iter().filter(|f| f.scripts.contains(&script)) {
            add_craft(&mut fonts, &mut cjk_chain, f);
        }
        let preferred = web_cjk_font(&needs.language);
        let mut downloaded: Vec<&(String, Arc<FontData>)> = needs.downloaded.iter().collect();
        downloaded.sort_by_key(|(name, _)| name != preferred);
        for (name, data) in downloaded {
            let key = format!("downloaded-{name}");
            fonts.font_data.insert(key.clone(), data.clone());
            cjk_chain.push(key);
        }
        let paths = cjk_candidates(&needs.language);
        let refs: Vec<(&str, u32)> = paths.iter().map(|(p, i)| (p.as_str(), *i)).collect();
        if let Some(fd) = try_load(&refs) {
            fonts.font_data.insert("sys-cjk".into(), Arc::new(fd));
            cjk_chain.push("sys-cjk".into());
        }
        if script != "Jpan" {
            for f in CRAFT_FONTS.iter().filter(|f| f.scripts.contains(&"Jpan") && !f.scripts.contains(&script)) {
                add_craft(&mut fonts, &mut cjk_chain, f);
            }
        }
    }
    for role in [UI, UI_BOLD, CELL, CELL_BOLD, CELL_ITALIC, CELL_BOLD_ITALIC, SERIF, MONO] {
        let paths = candidates(role);
        let refs: Vec<(&str, u32)> = paths.iter().map(|(p, i)| (p.as_str(), *i)).collect();
        let mut chain: Vec<String> = Vec::new();
        if let Some(fd) = try_load(&refs) {
            let key = format!("sys-{role}");
            fonts.font_data.insert(key.clone(), Arc::new(fd));
            chain.push(key);
        } else if role == UI_BOLD {
            chain.push(format!("sys-{UI}"));
        } else if role.starts_with("cell-") {
            chain.push(format!("sys-{CELL}"));
        }
        chain.extend(if role == MONO { base_mono.clone() } else { base_prop.clone() });
        chain.extend(cjk_chain.iter().cloned());
        chain.retain(|k| fonts.font_data.contains_key(k));
        fonts.families.insert(FontFamily::Name(role.into()), chain);
    }
    // Default proportional text in widgets uses the UI font.
    if let Some(ui) = fonts.families.get(&FontFamily::Name(UI.into())).cloned() {
        fonts.families.insert(FontFamily::Proportional, ui);
    }
    fonts
}

pub fn apply(ctx: &egui::Context, dark: bool) {
    let t = if dark { Tokens::dark() } else { Tokens::light() };
    let mut v = if dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.window;
    v.window_fill = t.menu_bg;
    v.extreme_bg_color = t.input_bg;
    v.selection.bg_fill = t.accent_soft;
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.hyperlink_color = t.accent;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.hovered.weak_bg_fill = t.hover;
    v.widgets.active.weak_bg_fill = t.pressed;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.input_border);
    v.window_corner_radius = egui::CornerRadius::same(10);
    v.menu_corner_radius = egui::CornerRadius::same(8);
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 16, spread: 0, color: t.shadow };
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(6.0, 3.0);
        s.spacing.interact_size.y = 22.0;
        s.text_styles.insert(egui::TextStyle::Body, ui_font(13.0));
        s.text_styles.insert(egui::TextStyle::Button, ui_font(13.0));
        s.text_styles.insert(egui::TextStyle::Small, ui_font(11.0));
        s.text_styles.insert(egui::TextStyle::Heading, ui_bold(18.0));
        s.text_styles.insert(egui::TextStyle::Monospace, FontId::new(12.0, FontFamily::Name(MONO.into())));
    });
}

/// Model colour → egui colour.
pub fn color32(rgb: [u8; 3]) -> Color32 {
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_detection() {
        assert!(has_cjk("売上 2026"));
        assert!(has_cjk("한글"));
        assert!(!has_cjk("Planilha1 ação"));
        assert!(is_cjk_language("ja-JP"));
        assert!(is_cjk_language("zh-TW"));
        assert!(!is_cjk_language("pt-BR"));
    }

    #[test]
    fn web_fonts_follow_the_language() {
        assert_eq!(web_cjk_font("ko-KR"), "cjk-ko.ttf");
        assert_eq!(web_cjk_font("zh-TW"), "cjk-zh-TW.ttf");
        assert_eq!(web_cjk_font("zh-CN"), "cjk-zh-CN.ttf");
        assert_eq!(web_cjk_font("ja-JP"), "cjk-ja.ttf");
    }

    #[test]
    fn downloaded_fonts_extend_every_family() {
        let data = Arc::new(FontData::from_static(&[0u8; 4]));
        let needs = FontNeeds { language: "ja-JP".into(), cjk: true, downloaded: vec![("cjk-ja.ttf".into(), data)] };
        let fonts = font_definitions(&needs);
        assert!(fonts.font_data.contains_key("downloaded-cjk-ja.ttf"));
        for family in [FontFamily::Proportional, FontFamily::Name(MONO.into()), FontFamily::Name(UI.into())] {
            assert!(fonts.families.get(&family).is_some_and(|f| f.iter().any(|n| n == "downloaded-cjk-ja.ttf")), "{family:?}");
        }
        // Without the CJK need the chain is not added.
        let plain = font_definitions(&FontNeeds::default());
        assert!(!plain.font_data.contains_key("downloaded-cjk-ja.ttf"));
    }

    #[test]
    fn the_font_of_the_interface_language_comes_first() {
        let data = Arc::new(FontData::from_static(&[0u8; 4]));
        let downloaded = vec![("cjk-ja.ttf".to_string(), data.clone()), ("cjk-ko.ttf".to_string(), data)];
        let position = |language: &str, font: &str| {
            let fonts = font_definitions(&FontNeeds { language: language.into(), cjk: true, downloaded: downloaded.clone() });
            let chain = fonts.families.get(&FontFamily::Name(UI.into())).cloned().unwrap_or_default();
            chain.iter().position(|n| n == font)
        };
        let (ko_in_ko, ja_in_ko) = (position("ko-KR", "downloaded-cjk-ko.ttf"), position("ko-KR", "downloaded-cjk-ja.ttf"));
        assert!(ko_in_ko.is_some() && ko_in_ko < ja_in_ko, "{ko_in_ko:?} {ja_in_ko:?}");
        let (ko_in_ja, ja_in_ja) = (position("ja-JP", "downloaded-cjk-ko.ttf"), position("ja-JP", "downloaded-cjk-ja.ttf"));
        assert!(ja_in_ja.is_some() && ja_in_ja < ko_in_ja, "{ja_in_ja:?} {ko_in_ja:?}");
    }

    #[test]
    fn the_font_key_ignores_the_language_until_cjk_is_needed() {
        let needs = |language: &str, cjk: bool| FontNeeds { language: language.into(), cjk, downloaded: vec![] };
        // Switching between non-CJK languages does not rebuild the fonts…
        assert_eq!(needs("pt-BR", false).key(), needs("de-DE", false).key());
        // …but with the CJK chain on, the language orders the faces.
        assert_ne!(needs("ja-JP", true).key(), needs("ko-KR", true).key());
        assert_ne!(needs("pt-BR", false).key(), needs("pt-BR", true).key());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn split_noto_files_are_ordered_by_language() {
        let first = |tag: &str| noto_otf_order(tag)[0];
        assert!(first("ko-KR").contains("kr"));
        assert!(first("zh-TW").contains("tc") && first("zh-HK").contains("tc"));
        assert!(first("zh-CN").contains("sc"));
        assert!(first("ja-JP").contains("jp") && first("pt-BR").contains("jp"));
        for tag in ["ko-KR", "zh-TW", "zh-CN", "ja-JP"] {
            let mut all = noto_otf_order(tag).to_vec();
            all.sort_unstable();
            all.dedup();
            assert_eq!(all.len(), 4, "{tag}");
        }
    }
}
