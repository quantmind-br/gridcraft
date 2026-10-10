use gridcraft_core::{CellRef, Value};
use serde_json::json;

use crate::Session;

fn s() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s
}

fn v(s: &Session, a: &str) -> Value {
    s.doc().unwrap().wb.active().unwrap().value(CellRef::parse(a).unwrap())
}

#[test]
fn enter_values_and_formulas() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "10"})).unwrap();
    s.execute("cell.set", json!({"cell": "A2", "input": "32"})).unwrap();
    s.execute("cell.set", json!({"cell": "A3", "input": "=sum(a1:a2"})).unwrap();
    assert_eq!(v(&s, "A3"), Value::Number(42.0));
    assert!(s.execute("cell.set", json!({"cell": "A4", "input": "=1+*2"})).is_err());
    s.execute("cell.set", json!({"cell": "B1", "input": "12%"})).unwrap();
    let d = s.doc().unwrap();
    let sh = d.wb.active().unwrap();
    assert_eq!(crate::display::cell_text(&d.wb, sh, CellRef::parse("B1").unwrap()), "12%");
}

#[test]
fn undo_redo() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "1"})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "input": "2"})).unwrap();
    s.execute("edit.undo", json!({})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(1.0));
    s.execute("edit.redo", json!({})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(2.0));
    assert!(s.doc().unwrap().is_dirty());
}

#[test]
fn copy_paste_shifts_formulas() {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1, 2], [3, 4]]})).unwrap();
    s.execute("cell.set", json!({"cell": "C1", "input": "=A1+B1"})).unwrap();
    s.execute("edit.copy", json!({"range": "C1"})).unwrap();
    s.execute("selection.set", json!({"range": "C2"})).unwrap();
    s.execute("edit.paste", json!({})).unwrap();
    assert_eq!(v(&s, "C2"), Value::Number(7.0));
    s.execute("edit.cut", json!({"range": "A1"})).unwrap();
    s.execute("selection.set", json!({"range": "E5"})).unwrap();
    s.execute("edit.paste", json!({})).unwrap();
    assert_eq!(v(&s, "E5"), Value::Number(1.0));
    let f = s.execute("cell.get", json!({"cell": "C1"})).unwrap();
    assert_eq!(f["formula"], "=E5+B1");
}

#[test]
fn insert_delete_rows_adjust() {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1], [2], [3]]})).unwrap();
    s.execute("cell.set", json!({"cell": "B1", "input": "=SUM(A1:A3)"})).unwrap();
    s.execute("home.insertRows", json!({"rows": "2:2"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "B1"})).unwrap()["formula"], "=SUM(A1:A4)");
    s.execute("cell.set", json!({"cell": "A2", "input": "10"})).unwrap();
    assert_eq!(v(&s, "B1"), Value::Number(16.0));
    s.execute("home.deleteRows", json!({"rows": "1:1"})).unwrap();
    assert_eq!(v(&s, "B1"), Value::Empty);
    assert_eq!(v(&s, "A1"), Value::Number(10.0));
}

#[test]
fn fill_series() {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1], [3]]})).unwrap();
    s.execute("edit.autoFill", json!({"source": "A1:A2", "target": "A1:A6"})).unwrap();
    assert_eq!(v(&s, "A6"), Value::Number(11.0));
    s.execute("cell.set", json!({"cell": "B1", "input": "Mon"})).unwrap();
    s.execute("edit.autoFill", json!({"source": "B1", "target": "B1:B3"})).unwrap();
    assert_eq!(v(&s, "B3"), Value::from("Wed"));
    s.execute("cell.set", json!({"cell": "C1", "input": "Item 9"})).unwrap();
    s.execute("edit.autoFill", json!({"source": "C1", "target": "C1:C2"})).unwrap();
    assert_eq!(v(&s, "C2"), Value::from("Item 10"));
}

#[test]
fn sort_and_filter() {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [["Name", "Score"], ["b", 2], ["c", 3], ["a", 1]]})).unwrap();
    s.execute("selection.set", json!({"cell": "B2"})).unwrap();
    s.execute("data.sortDescending", json!({})).unwrap();
    assert_eq!(v(&s, "A2"), Value::from("c"));
    assert_eq!(v(&s, "A1"), Value::from("Name"));
    s.execute("data.filter", json!({})).unwrap();
    let r = s.execute("data.filterBy", json!({"column": "B", "custom": {"op": ">", "value": "1"}})).unwrap();
    assert_eq!(r["hiddenRows"], 1);
}

#[test]
fn formatting_and_styles() {
    let mut s = s();
    s.execute("selection.set", json!({"range": "A1:B2"})).unwrap();
    s.execute("home.bold", json!({})).unwrap();
    s.execute("home.fillColor", json!({"color": "#FFFF00"})).unwrap();
    s.execute("home.mergeCenter", json!({})).unwrap();
    let c = s.execute("cell.get", json!({"cell": "B2"})).unwrap();
    assert_eq!(c["style"]["font"]["bold"], true);
    assert_eq!(c["merge"], "A1:B2");
    s.execute("home.cellStyle", json!({"range": "D1", "name": "Good"})).unwrap();
}

#[test]
fn tables_and_structured_refs() {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [["Item", "Qty"], ["x", 2], ["y", 5]]})).unwrap();
    s.execute("insert.table", json!({"range": "A1:B3"})).unwrap();
    s.execute("cell.set", json!({"cell": "D1", "input": "=SUM(Table1[Qty])"})).unwrap();
    assert_eq!(v(&s, "D1"), Value::Number(7.0));
    s.execute("table.totalRow", json!({"on": true, "table": "Table1"})).unwrap();
    assert_eq!(v(&s, "B4"), Value::Number(7.0));
}

#[test]
fn sheets() {
    let mut s = s();
    s.execute("home.insertSheet", json!({})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "input": "5"})).unwrap();
    s.execute("sheet.rename", json!({"name": "Data"})).unwrap();
    s.execute("sheet.activate", json!({"sheet": 0})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "input": "=Data!A1*2"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(10.0));
    s.execute("sheet.rename", json!({"sheet": "Data", "name": "My Data"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["formula"], "='My Data'!A1*2");
}

#[test]
fn xlsx_roundtrip_through_engine() {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [["a", 1], ["b", 2]]})).unwrap();
    s.execute("cell.set", json!({"cell": "B3", "input": "=SUM(B1:B2)"})).unwrap();
    let r = s.execute("file.saveBytes", json!({"format": "xlsx"})).unwrap();
    let b64 = r["base64"].as_str().unwrap().to_string();
    s.execute("file.open", json!({"name": "x.xlsx", "base64": b64})).unwrap();
    assert_eq!(v(&s, "B3"), Value::Number(3.0));
}

#[test]
fn samples_build() {
    for (name, _) in crate::sample::SAMPLES {
        let wb = crate::sample::build(name).unwrap_or_else(|| panic!("sample {name}"));
        assert!(!wb.sheets[0].cells.is_empty());
    }
}

#[test]
fn every_command_survives_empty_params() {
    let mut s = Session::new();
    s.execute("file.new", json!({"sample": "sales"})).unwrap();
    for spec in crate::command_specs() {
        if matches!(spec.id, "file.close" | "file.open" | "file.save" | "file.saveAs" | "file.exportCsv" | "file.exportHtml" | "insert.picture") {
            continue;
        }
        let _ = s.execute(spec.id, json!({}));
        let _ = s.execute(spec.id, json!([1, 2]));
        let _ = s.execute(spec.id, json!({"range": "ZZZ999999999", "cell": "", "sheet": 99}));
    }
}

#[test]
fn parity_counts() {
    let (done, total) = crate::catalog::parity();
    assert!(total > 250);
    assert!(done > 150, "parity {done}/{total}");
}

#[test]
fn ink_strokes_and_ink_to_shape() {
    let mut s = s();
    // A rough closed box.
    let pts: Vec<[f64; 2]> = vec![[100.0, 100.0], [200.0, 102.0], [201.0, 180.0], [99.0, 181.0], [101.0, 104.0]];
    s.execute("draw.stroke", json!({"points": pts})).unwrap();
    let sh = s.doc().unwrap().wb.active().unwrap().clone();
    assert_eq!(sh.shapes.len(), 1);
    assert_eq!(sh.shapes[0].kind, gridcraft_model::ShapeKind::Ink);
    let r = s.execute("draw.inkToShape", json!({})).unwrap();
    assert_eq!(r["kind"], "Rectangle");
    // An open stroke becomes a line.
    s.execute("draw.stroke", json!({"points": [[0.0, 0.0], [50.0, 10.0], [120.0, 30.0]]})).unwrap();
    assert_eq!(s.execute("draw.inkToShape", json!({})).unwrap()["kind"], "Line");
    assert!(s.execute("draw.stroke", json!({"points": [[1.0, 1.0]]})).is_err());
}

fn set_locale(s: &mut Session, p: serde_json::Value) {
    s.execute("app.setLocale", p).unwrap();
}

fn text(s: &Session, a: &str) -> String {
    let d = s.doc().unwrap();
    crate::display::cell_text(&d.wb, d.wb.active().unwrap(), CellRef::parse(a).unwrap())
}

#[test]
fn regional_input_and_display_follow_the_region() {
    let mut s = s();
    set_locale(&mut s, json!({"regionalFormat": "pt-BR", "formulaLanguage": "en-US"}));
    s.execute("cell.set", json!({"cell": "A1", "inputLocal": "=SUM(1,5;2)"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(3.5));
    assert_eq!(text(&s, "A1"), "3,5");
    let r = s.execute("cell.get", json!({"cell": "A1"})).unwrap();
    assert_eq!(r["formula"], "=SUM(1.5,2)");
    assert_eq!(r["formulaLocal"], "=SUM(1,5;2)");
    // Programmatic input stays canonical.
    s.execute("cell.set", json!({"cell": "A2", "input": "=SUM(1.5,2)"})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(3.5));
}

#[test]
fn invalid_locale_is_rejected_without_partial_change() {
    let mut s = s();
    set_locale(&mut s, json!({"regionalFormat": "de-DE"}));
    let before = s.locale();
    let err = s.execute("app.setLocale", json!({"regionalFormat": "pt-BR", "uiLanguage": "xx-ZZ"})).unwrap_err();
    assert_eq!(err.code(), "invalid-locale");
    assert_eq!(s.locale().regional.tag, before.regional.tag);
    assert_eq!(s.locale().regional.tag, "de-DE");
    assert_eq!(s.prefs.regional_format, "de-DE");
}

#[test]
fn manual_mode_leaves_values_stale_after_a_locale_change() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "=1.5&\"\""})).unwrap();
    assert_eq!(v(&s, "A1"), Value::text("1.5"));
    s.execute("formulas.calculationOptions", json!({"mode": "manual"})).unwrap();
    set_locale(&mut s, json!({"regionalFormat": "pt-BR", "formulaLanguage": "en-US"}));
    assert_eq!(v(&s, "A1"), Value::text("1.5"));
    s.execute("formulas.calculateNow", json!({})).unwrap();
    assert_eq!(v(&s, "A1"), Value::text("1,5"));
}

#[test]
fn automatic_mode_recalculates_after_a_locale_change() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "=1.5&\"\""})).unwrap();
    set_locale(&mut s, json!({"regionalFormat": "pt-BR", "formulaLanguage": "en-US"}));
    assert_eq!(v(&s, "A1"), Value::text("1,5"));
}

#[test]
fn a_locale_change_does_not_dirty_a_clean_document() {
    let mut s = s();
    assert!(!s.doc().unwrap().is_dirty());
    set_locale(&mut s, json!({"regionalFormat": "pt-BR", "formulaLanguage": "en-US"}));
    assert!(!s.doc().unwrap().is_dirty());
}

#[test]
fn csv_export_uses_the_region_list_separator_and_decimal() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "1.5"})).unwrap();
    s.execute("cell.set", json!({"cell": "B1", "input": "2"})).unwrap();
    set_locale(&mut s, json!({"regionalFormat": "pt-BR", "formulaLanguage": "en-US"}));
    let bytes = crate::io::save_bytes(&s.doc().unwrap().wb, "book.csv").unwrap();
    assert_eq!(String::from_utf8_lossy(&bytes).trim_end(), "1,5;2");
    let bytes = crate::io::save_bytes_with(&s.doc().unwrap().wb, "book.csv", Some(b'\t')).unwrap();
    assert_eq!(String::from_utf8_lossy(&bytes).trim_end(), "1,5\t2");
}

#[test]
fn error_codes_are_stable() {
    use crate::EngineError as E;
    assert_eq!(E::UnknownCommand("x".into()).code(), "unknown-command");
    assert_eq!(E::Disabled("x".into(), "r".into()).code(), "command-disabled");
    assert_eq!(E::BadParams { cmd: "x".into(), msg: "m".into() }.code(), "bad-params");
    assert_eq!(E::NoDocument.code(), "no-document");
    assert_eq!(E::Other("m".into()).code(), "other");
    assert_eq!(E::InvalidFormula("m".into()).code(), "invalid-formula");
    assert_eq!(E::InvalidLocale("m".into()).code(), "invalid-locale");
    assert_eq!(E::Disabled("x".into(), "r".into()).args(), vec![("command", "x".to_string()), ("reason", "r".to_string())]);
    assert_eq!(E::BadParams { cmd: "x".into(), msg: "m".into() }.args(), vec![("command", "x".to_string()), ("message", "m".to_string())]);
    assert_eq!(E::Other("m".into()).to_string(), "m");
}

#[test]
fn formula_text_round_trips_through_the_region_dialect() {
    let region = gridcraft_locale::region("de-DE").unwrap();
    let d = gridcraft_locale::Dialect { names: gridcraft_locale::language("en-US").unwrap(), regional: region };
    let local = crate::locale::to_local_formula("=SUM(1.5,A1)", &d);
    assert_eq!(local, "=SUM(1,5;A1)");
    assert_eq!(crate::locale::from_local_formula(&local, &d).unwrap(), "=SUM(1.5,A1)");
    assert!(crate::locale::from_local_formula("=1+*2", &d).is_err());
}

#[test]
fn languages_and_international_payloads() {
    let mut s = s();
    let l = s.execute("app.languages", json!({})).unwrap();
    assert!(l["languages"].as_array().unwrap().iter().any(|x| x["tag"] == "en-US"));
    assert!(l["regions"].as_array().unwrap().iter().any(|x| x["tag"] == "pt-BR"));
    set_locale(&mut s, json!({"regionalFormat": "de-DE", "formulaLanguage": "en-US"}));
    let i = s.execute("app.getInternational", json!({})).unwrap();
    assert_eq!(i["decimal"], ",");
}

#[test]
fn pt_br_ui_and_formula_language() {
    let mut s = Session::new();
    s.set_locale(crate::LocalePrefs { ui_language: Some("pt-BR".into()), regional_format: Some("pt-BR".into()), ..Default::default() }).unwrap();
    s.new_workbook();
    assert_eq!(s.doc().unwrap().wb.sheets[0].name, "Planilha1");
    s.execute("cell.set", json!({"cell": "A1", "inputLocal": "=SOMA(1,5;2)"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["formula"], "=SUM(1.5,2)");
    assert_eq!(text(&s, "A1"), "3,5");
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["formulaLocal"], "=SOMA(1,5;2)");
    s.execute("home.insertSheet", json!({})).unwrap();
    let d = s.doc().unwrap();
    assert!(d.wb.sheets.iter().any(|sh| sh.name == "Planilha2"));
}

#[test]
fn subtotal_grand_row_label() {
    let mut s = s();
    for (a, x) in [("A1", "g"), ("B1", "n"), ("A2", "a"), ("B2", "1"), ("A3", "b"), ("B3", "2")] {
        s.execute("cell.set", json!({"cell": a, "input": x})).unwrap();
    }
    s.execute("data.subtotal", json!({"range": "A1:B3", "groupBy": "A", "columns": ["B"]})).unwrap();
    let grand_sum = (0..8).map(|r| text(&s, &format!("A{}", r + 1))).find(|t| t.starts_with("Grand"));
    assert_eq!(grand_sum.as_deref(), Some("Grand Total"));
    let mut s2 = self::s();
    for (a, x) in [("A1", "g"), ("B1", "n"), ("A2", "a"), ("B2", "1"), ("A3", "b"), ("B3", "2")] {
        s2.execute("cell.set", json!({"cell": a, "input": x})).unwrap();
    }
    s2.execute("data.subtotal", json!({"range": "A1:B3", "groupBy": "A", "columns": ["B"], "function": "count"})).unwrap();
    let grand_count = (0..8).map(|r| text(&s2, &format!("A{}", r + 1))).find(|t| t.starts_with("Grand"));
    assert_eq!(grand_count.as_deref(), Some("Grand Count"));
}

fn session_in(prefs: crate::LocalePrefs) -> Session {
    let mut s = Session::new();
    s.set_locale(prefs).unwrap();
    s.new_workbook();
    s
}

fn prefs(ui: &str, region: &str) -> crate::LocalePrefs {
    crate::LocalePrefs { ui_language: Some(ui.into()), regional_format: Some(region.into()), ..Default::default() }
}

fn region_locale(tag: &str) -> gridcraft_locale::Locale {
    let inv = gridcraft_locale::INVARIANT;
    gridcraft_locale::Locale::new(inv.ui, inv.formula, *gridcraft_locale::region(tag).unwrap())
}

#[test]
fn csv_open_splits_on_the_region_list_separator() {
    let pt = region_locale("pt-BR");
    let (wb, _) = crate::io::open_bytes("a.csv", b"1,5\r\n", &pt).unwrap();
    let sh = wb.sheet(0).unwrap();
    assert_eq!(sh.value(CellRef::new(0, 0)), Value::Number(1.5));
    assert!(sh.value(CellRef::new(0, 1)).is_empty());
    let (wb, _) = crate::io::open_bytes("a.csv", b"1,5;2,5\r\n", &pt).unwrap();
    let sh = wb.sheet(0).unwrap();
    assert_eq!((sh.value(CellRef::new(0, 0)), sh.value(CellRef::new(0, 1))), (Value::Number(1.5), Value::Number(2.5)));
    // En-US is unchanged: the comma separates fields.
    let (wb, _) = crate::io::open_bytes("a.csv", b"1,5\r\n", &gridcraft_locale::INVARIANT).unwrap();
    let sh = wb.sheet(0).unwrap();
    assert_eq!((sh.value(CellRef::new(0, 0)), sh.value(CellRef::new(0, 1))), (Value::Number(1.0), Value::Number(5.0)));
    // An explicit delimiter wins over detection.
    let (wb, _) = crate::io::open_bytes_with("a.csv", b"1,5\r\n", &gridcraft_locale::INVARIANT, Some(b';')).unwrap();
    assert_eq!(wb.sheet(0).unwrap().value(CellRef::new(0, 0)), Value::text("1,5"));
}

#[test]
fn delimiter_params_are_validated() {
    let mut s = s();
    for bad in [json!("\""), json!("\n"), json!("\r"), json!("\0"), json!(";;"), json!(""), json!(5), json!(true)] {
        let err = s.execute("file.saveBytes", json!({"format": "csv", "delimiter": bad})).unwrap_err();
        assert_eq!(err.code(), "bad-params", "{bad}");
    }
    assert!(s.execute("file.saveBytes", json!({"format": "csv", "delimiter": "\t"})).is_ok());
    let b64 = crate::io::base64_encode(b"a;b\r\n1;2\r\n");
    let err = s.execute("file.open", json!({"name": "x.csv", "base64": b64, "delimiter": "\""})).unwrap_err();
    assert_eq!(err.code(), "bad-params");
    s.execute("file.open", json!({"name": "x.csv", "base64": b64, "delimiter": ";"})).unwrap();
    assert_eq!(v(&s, "B2"), Value::Number(2.0));
}

#[test]
fn text_to_columns_keeps_numbers_typed() {
    let mut s = session_in(prefs("en-US", "pt-BR"));
    s.execute("cell.set", json!({"cell": "A1", "input": "1234.5"})).unwrap();
    s.execute("data.textToColumns", json!({"range": "A1", "other": "|"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(1234.5));
}

#[test]
fn validation_local_conversion_and_typed_lists() {
    let mut s = session_in(prefs("en-US", "pt-BR"));
    s.execute("cell.set", json!({"cell": "B1", "input": "1.5"})).unwrap();
    s.execute("cell.set", json!({"cell": "B2", "input": "2"})).unwrap();
    s.execute("data.validation", json!({"range": "A1", "type": "list", "formula1": "$B$1:$B$2"})).unwrap();
    assert_eq!(s.execute("data.validate", json!({"cell": "A1", "inputLocal": "1,5"})).unwrap()["ok"], true);
    assert_eq!(s.execute("data.validate", json!({"cell": "A1", "inputLocal": "1,6"})).unwrap()["ok"], false);
    // A literal list may use the region's list separator…
    s.execute("data.validation", json!({"range": "A2", "type": "list", "formula1Local": "Sim;Não"})).unwrap();
    assert_eq!(s.execute("data.validate", json!({"cell": "A2", "inputLocal": "Não"})).unwrap()["ok"], true);
    // …but a broken formula in any other definition is an error, not a literal.
    let err = s.execute("data.validation", json!({"range": "A3", "type": "custom", "formula1Local": "=1+*2"})).unwrap_err();
    assert_eq!(err.code(), "invalid-formula");
    let err = s.execute("data.validation", json!({"range": "A3", "type": "list", "formula1Local": "=1+*2"})).unwrap_err();
    assert_eq!(err.code(), "invalid-formula");
}

fn pivot_numbers(s: &Session) -> f64 {
    let d = s.doc().unwrap();
    let sh = d.wb.active().unwrap();
    sh.cells.iter().filter_map(|(_, c)| c.value.as_f64()).sum()
}

#[test]
fn pivot_blank_filter_survives_a_language_switch() {
    let mut s = s();
    for (a, x) in [("A1", "k"), ("B1", "v"), ("A2", "x"), ("B2", "1"), ("B3", "2")] {
        s.execute("cell.set", json!({"cell": a, "input": x})).unwrap();
    }
    s.execute("insert.pivotTable", json!({"source": "A1:B3", "rows": ["k"], "values": [{"field": "v"}]})).unwrap();
    s.execute("pivot.filter", json!({"field": "k", "selected": [null]})).unwrap();
    let before = pivot_numbers(&s);
    assert_eq!(before, 4.0);
    set_locale(&mut s, json!({"uiLanguage": "pt-PT", "regionalFormat": "pt-PT"}));
    s.execute("pivot.refresh", json!({})).unwrap();
    assert_eq!(pivot_numbers(&s), before);
    set_locale(&mut s, json!({"uiLanguage": "de-DE", "regionalFormat": "de-DE"}));
    s.execute("pivot.refresh", json!({})).unwrap();
    assert_eq!(pivot_numbers(&s), before);
}

#[test]
fn pivot_auto_caption_follows_the_function_across_languages() {
    let mut s = s();
    for (a, x) in [("A1", "k"), ("B1", "v"), ("A2", "x"), ("B2", "1")] {
        s.execute("cell.set", json!({"cell": a, "input": x})).unwrap();
    }
    s.execute("insert.pivotTable", json!({"source": "A1:B2", "rows": ["k"], "values": [{"field": "v"}]})).unwrap();
    set_locale(&mut s, json!({"uiLanguage": "de-DE", "regionalFormat": "de-DE"}));
    s.execute("pivot.valueSettings", json!({"index": 0, "func": "count"})).unwrap();
    let name = s.execute("pivot.list", json!({})).unwrap().to_string();
    assert!(!name.contains("Sum of v"), "{name}");
    assert!(name.contains("Anzahl von v"), "{name}");
}

#[test]
fn custom_separators_are_restricted() {
    let mut s = s();
    for bad in ["+", "a", "1", "\"", "(", "\n", "ab", ""] {
        let err = s.execute("app.setLocale", json!({"useSystemSeparators": false, "decimalSeparator": bad})).unwrap_err();
        assert!(matches!(err.code(), "invalid-locale" | "bad-params"), "{bad}");
    }
    assert_eq!(s.prefs.decimal_separator, ".");
    s.execute("app.setLocale", json!({"useSystemSeparators": false, "decimalSeparator": ",", "thousandsSeparator": "\u{2019}"})).unwrap();
    assert_eq!(s.locale().regional.decimal, ',');
}

#[test]
fn formula_length_is_capped() {
    let mut s = s();
    let long = format!("={}1", "1+".repeat(5000));
    assert_eq!(s.execute("cell.set", json!({"cell": "A1", "input": long})).unwrap_err().code(), "invalid-formula");
    assert_eq!(s.execute("cell.set", json!({"cell": "A1", "inputLocal": long})).unwrap_err().code(), "invalid-formula");
    assert!(s.execute("formulas.evaluate", json!({"formulaLocal": long})).is_err());
    let d = gridcraft_locale::INVARIANT.dialect();
    assert!(crate::locale::from_local_formula(&long, &d).is_err());
    assert!(s.execute("cell.set", json!({"cell": "A1", "input": "=1+1"})).is_ok());
}

#[test]
fn constant_errors_round_trip_in_the_formula_language() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    s.execute("cell.set", json!({"cell": "A1", "input": "#VALUE!"})).unwrap();
    assert!(v(&s, "A1").is_error());
    let local = s.execute("cell.get", json!({"cell": "A1"})).unwrap()["inputLocal"].as_str().unwrap().to_string();
    assert_ne!(local, "#VALUE!");
    s.execute("cell.set", json!({"cell": "B1", "inputLocal": local})).unwrap();
    assert_eq!(v(&s, "B1"), v(&s, "A1"));
}

#[test]
fn csv_import_decides_cell_types_by_region_once() {
    let mut s = session_in(prefs("en-US", "pt-BR"));
    s.execute("data.getData", json!({"text": "a;b\r\n1,5;1.5\r\n", "local": true})).unwrap();
    let reg = gridcraft_locale::region("pt-BR").unwrap();
    let expect = gridcraft_core::parse::parse_input_in("1.5", gridcraft_core::DateSystem::D1900, reg, gridcraft_locale::INVARIANT.formula).value;
    assert_eq!(v(&s, "A2"), Value::Number(1.5));
    assert_eq!(v(&s, "B2"), expect);
}

#[test]
fn table_resize_keeps_columns_across_a_language_switch() {
    let mut s = s();
    for (a, x) in [("A1", "a"), ("B1", "b"), ("A2", "1"), ("B2", "2")] {
        s.execute("cell.set", json!({"cell": a, "input": x})).unwrap();
    }
    s.execute("insert.table", json!({"range": "A1:B2", "header": true})).unwrap();
    s.execute("table.resize", json!({"range": "A1:C2"})).unwrap();
    let name = |s: &Session| s.doc().unwrap().wb.sheets[0].tables[0].columns[2].name.clone();
    let before = name(&s);
    set_locale(&mut s, json!({"uiLanguage": "de-DE", "regionalFormat": "de-DE"}));
    s.execute("table.resize", json!({"range": "A1:C2"})).unwrap();
    assert_eq!(name(&s), before);
}

#[test]
fn find_and_replace_work_on_what_the_formula_bar_shows() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    s.execute("cell.set", json!({"cell": "A1", "inputLocal": "=SUMME(B1;C1)"})).unwrap();
    s.execute("cell.set", json!({"cell": "A2", "inputLocal": "1,5"})).unwrap();
    assert_eq!(s.execute("edit.find", json!({"what": "SUMME", "lookIn": "formulas"})).unwrap()["cell"], "A1");
    s.execute("edit.replace", json!({"what": "SUMME", "with": "MITTELWERT"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["formula"], "=AVERAGE(B1,C1)");
    s.execute("edit.replace", json!({"what": "1,5", "with": "2,5"})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(2.5));
    // En-US keeps its behaviour.
    let mut e = self::s();
    e.execute("cell.set", json!({"cell": "A1", "input": "=SUM(B1,C1)"})).unwrap();
    e.execute("edit.replace", json!({"what": "SUM", "with": "AVERAGE"})).unwrap();
    assert_eq!(e.execute("cell.get", json!({"cell": "A1"})).unwrap()["formula"], "=AVERAGE(B1,C1)");
}

#[test]
fn the_journal_is_canonical_and_skips_preferences() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    s.execute("app.options", json!({})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "inputLocal": "=SUMME(1,5;2)"})).unwrap();
    s.execute("cell.set", json!({"cell": "A2", "inputLocal": "1,5"})).unwrap();
    s.execute("home.numberFormat", json!({"range": "A2", "numberFormatLocal": "#.##0,00"})).unwrap();
    assert!(s.journal.iter().all(|(id, _)| id != "app.setLocale" && id != "app.options"));
    let recorded: Vec<(String, serde_json::Value)> = s.journal.clone();
    let mut replay = self::s();
    for (id, p) in &recorded {
        replay.execute(id, p.clone()).unwrap();
    }
    assert_eq!(v(&replay, "A1"), Value::Number(3.5));
    assert_eq!(v(&replay, "A2"), Value::Number(1.5));
    let recorded_text = serde_json::to_string(&recorded).unwrap();
    assert!(!recorded_text.contains("Local"), "{recorded_text}");
    assert_eq!(s.execute("formulas.recentlyUsed", json!({})).unwrap()[0], "SUM");
}

#[test]
fn watch_window_rows_carry_local_fields() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    s.execute("cell.set", json!({"cell": "A1", "inputLocal": "=SUMME(1,5;2)"})).unwrap();
    let rows = s.execute("formulas.watchWindow", json!({"add": "A1"})).unwrap();
    assert_eq!(rows[0]["formula"], "=SUM(1.5,2)");
    assert_eq!(rows[0]["formulaLocal"], "=SUMME(1,5;2)");
    assert_eq!(rows[0]["text"], "3,5");
}

#[test]
fn undo_and_redo_keep_the_session_locale() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "1.5"})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "input": "2.5"})).unwrap();
    set_locale(&mut s, json!({"regionalFormat": "de-DE", "formulaLanguage": "en-US"}));
    s.execute("edit.undo", json!({})).unwrap();
    assert_eq!(s.doc().unwrap().wb.locale.regional.tag, "de-DE");
    assert_eq!(text(&s, "A1"), "1,5");
    s.execute("edit.redo", json!({})).unwrap();
    assert_eq!(text(&s, "A1"), "2,5");
}

#[test]
fn system_locale_applies_only_to_system_prefs() {
    let mut s = Session::new();
    s.set_system_locale("de_DE.UTF-8");
    assert_eq!(s.locale().regional.tag, "de-DE");
    assert_eq!(s.locale().ui.tag, "de-DE");
    s.new_workbook();
    assert_eq!(s.doc().unwrap().wb.locale.regional.tag, "de-DE");
    set_locale(&mut s, json!({"regionalFormat": "pt-BR"}));
    s.set_system_locale("fr-FR");
    assert_eq!(s.locale().regional.tag, "pt-BR");
    assert_eq!(s.locale().ui.tag, "fr-FR");
    s.take_ui_requests();
    s.set_system_locale("fr-FR");
    assert!(s.take_ui_requests().is_empty());
}

#[test]
fn stale_saved_locale_prefs_are_repaired_field_by_field() {
    let mut s = Session::new();
    // As loaded from an older or hand-edited prefs file: a language this build lacks and
    // colliding separators, next to a valid region.
    s.prefs.ui_language = "xx-XX".into();
    s.prefs.regional_format = "de-DE".into();
    s.prefs.use_system_separators = false;
    s.prefs.decimal_separator = ",".into();
    s.prefs.thousands_separator = ",".into();
    s.set_system_locale("pt_BR.UTF-8");
    assert_eq!(s.prefs.ui_language, "system");
    assert_eq!(s.prefs.regional_format, "de-DE");
    assert!(s.prefs.use_system_separators);
    assert_eq!(s.locale().ui.tag, "pt-BR");
    assert_eq!(s.locale().regional.tag, "de-DE");
    // Later partial changes validate against the repaired prefs instead of failing.
    set_locale(&mut s, json!({"formulaLanguage": "en-US"}));
    assert_eq!(s.locale().formula.tag, "en-US");
}

#[test]
fn content_templates_expand_placeholders_safely() {
    let de = gridcraft_locale::language("de-DE").unwrap();
    let loc = gridcraft_locale::Locale::new(de, de, gridcraft_locale::INVARIANT.regional);
    assert_eq!(crate::locale::content_fmt(&loc, "subtotal_grand", &[("label", "Mittelwert")]), "Gesamtmittelwert");
    // A placeholder without an argument stays verbatim.
    assert_eq!(crate::locale::content_fmt(&loc, "subtotal_group", &[("name", "x")]), "x {label}");
    // Values are inserted literally, never re-scanned.
    assert_eq!(crate::locale::content_fmt(&loc, "subtotal_group", &[("name", "{label}"), ("label", "T")]), "{label} T");
}

#[test]
fn subtotal_labels_follow_the_ui_language() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    for (a, x) in [("A1", "g"), ("B1", "n"), ("A2", "a"), ("B2", "1"), ("A3", "b"), ("B3", "2")] {
        s.execute("cell.set", json!({"cell": a, "input": x})).unwrap();
    }
    s.execute("data.subtotal", json!({"range": "A1:B3", "groupBy": "A", "columns": ["B"], "function": "average"})).unwrap();
    let labels: Vec<String> = (1..=8).map(|r| text(&s, &format!("A{r}"))).collect();
    assert!(labels.iter().any(|t| t == "Gesamtmittelwert"), "{labels:?}");
    let mut s = session_in(prefs("de-DE", "de-DE"));
    for (a, x) in [("A1", "g"), ("B1", "n"), ("A2", "a"), ("B2", "1"), ("A3", "b"), ("B3", "2")] {
        s.execute("cell.set", json!({"cell": a, "input": x})).unwrap();
    }
    s.execute("data.subtotal", json!({"range": "A1:B3", "groupBy": "A", "columns": ["B"]})).unwrap();
    let labels: Vec<String> = (1..=8).map(|r| text(&s, &format!("A{r}"))).collect();
    assert!(labels.iter().any(|t| t == "Gesamtergebnis"), "{labels:?}");
}

#[test]
fn international_reports_the_system_locale_the_system_prefs_resolve_to() {
    let mut s = Session::new();
    s.set_system_locale("de_DE.UTF-8");
    set_locale(&mut s, json!({"regionalFormat": "en-US"}));
    let intl = s.execute("app.getInternational", json!({})).unwrap();
    assert_eq!(intl["systemLocale"], json!("de_DE.UTF-8"));
    assert_eq!(intl["systemLanguage"], json!("de-DE"));
    assert_eq!(intl["systemRegion"], json!("de-DE"));
    // The stored region is independent of the system one.
    assert_eq!(intl["regionalFormat"], json!("en-US"));
    assert_eq!(intl["decimal"], json!("."));
}

#[test]
fn formula_auditing_reports_local_text_next_to_canonical() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "=SUM(1.5,2)+\"x\""})).unwrap();
    set_locale(&mut s, json!({"uiLanguage": "de-DE", "regionalFormat": "de-DE"}));
    let local_error = s.locale().formula.local_error("#VALUE!").to_string();
    assert_eq!(local_error, "#WERT!");

    let report = s.execute("formulas.errorChecking", json!({})).unwrap();
    let row = &report["errors"][0];
    assert_eq!(row["formula"], json!("=SUM(1.5,2)+\"x\""));
    assert_eq!(row["formulaLocal"], json!("=SUMME(1,5;2)+\"x\""));
    assert_eq!(row["error"], json!("#VALUE!"));
    assert_eq!(row["errorLocal"], json!("#WERT!"));

    let steps = s.execute("formulas.evaluateFormula", json!({"cell": "A1"})).unwrap();
    assert_eq!(steps["formula"], json!("=SUM(1.5,2)+\"x\""));
    assert_eq!(steps["formulaLocal"], json!("=SUMME(1,5;2)+\"x\""));
    assert_eq!(steps["resultLocal"], json!("#WERT!"));
    let steps = steps["steps"].as_array().unwrap();
    assert!(steps.iter().any(|st| st["expression"] == json!("SUM(1.5,2)") && st["expressionLocal"] == json!("SUMME(1,5;2)")), "{steps:?}");
    // A step's value is spelled in the region too.
    assert!(steps.iter().any(|st| st["expressionLocal"] == json!("SUMME(1,5;2)") && st["valueLocal"] == json!("3,5")), "{steps:?}");

    // The formula to evaluate may be typed in the session's language.
    let typed = s.execute("formulas.evaluateFormula", json!({"formulaLocal": "=SUMME(1,5;2)"})).unwrap();
    assert_eq!(typed["formula"], json!("=SUM(1.5,2)"));
    assert_eq!(typed["resultLocal"], json!("3,5"));
}

fn cond_formats(s: &Session) -> Vec<gridcraft_model::CfRule> {
    s.doc().unwrap().wb.sheets[0].cond_formats.iter().map(|c| c.rule.clone()).collect()
}

#[test]
fn conditional_format_rules_are_journaled_canonically() {
    let mut s = session_in(prefs("pt-BR", "pt-BR"));
    s.execute("home.conditionalFormat", json!({"range": "A1", "rule": {"type": "cellIs", "operator": "greater", "valueLocal": "1,5"}})).unwrap();
    s.execute("home.conditionalFormat", json!({"range": "A2", "rule": {"type": "cellIs", "operator": "equal", "valueLocal": "Sim"}})).unwrap();
    s.execute("home.conditionalFormat", json!({"range": "A3", "rule": {"type": "expression", "formulaLocal": "=SE(A1>1,5;1;0)"}})).unwrap();
    let recorded = serde_json::to_string(&s.journal).unwrap();
    assert!(!recorded.contains("Local"), "{recorded}");
    let mut replay = self::s();
    for (id, p) in s.journal.clone() {
        replay.execute(&id, p).unwrap();
    }
    assert_eq!(cond_formats(&replay), cond_formats(&s));
    assert!(cond_formats(&replay).iter().any(|r| matches!(r, gridcraft_model::CfRule::CellIs { a, .. } if a == "1.5")));
}

#[test]
fn conditional_format_accepts_numeric_local_operands() {
    let mut s = session_in(prefs("pt-BR", "pt-BR"));
    s.execute(
        "home.conditionalFormat",
        json!({"range": "A1", "rule": {"type": "cellIs", "operator": "between", "valueLocal": 1.5, "value2Local": 3}}),
    )
    .unwrap();
    assert!(
        matches!(&cond_formats(&s)[0], gridcraft_model::CfRule::CellIs { a, b: Some(b), .. } if a == "1.5" && b == "3"),
        "{:?}",
        cond_formats(&s)
    );
    let mut replay = self::s();
    for (id, p) in s.journal.clone() {
        replay.execute(&id, p).unwrap();
    }
    assert_eq!(cond_formats(&replay), cond_formats(&s));
}

#[test]
fn region_currency_input_replays_in_any_locale() {
    let mut s = session_in(prefs("pt-BR", "pt-BR"));
    s.execute("cell.set", json!({"cell": "A1", "inputLocal": "R$ 3,5"})).unwrap();
    s.execute("range.setValues", json!({"range": "B1", "values": [["R$ 4,25", "1,5"]], "local": true})).unwrap();
    let fmt = |s: &mut Session, a: &str| s.execute("cell.get", json!({"cell": a})).unwrap()["numberFormat"].as_str().unwrap().to_string();
    let want = fmt(&mut s, "A1");
    assert!(want.contains("R$"), "{want}");
    let mut replay = self::s();
    for (id, p) in s.journal.clone() {
        replay.execute(&id, p).unwrap();
    }
    assert_eq!(v(&replay, "A1"), Value::Number(3.5));
    assert_eq!(v(&replay, "B1"), Value::Number(4.25));
    assert_eq!(v(&replay, "C1"), Value::Number(1.5));
    for a in ["A1", "B1", "C1"] {
        assert_eq!(fmt(&mut replay, a), fmt(&mut s, a), "{a}");
    }
}

#[test]
fn arrays_are_spelled_as_local_array_constants() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    let r = s.execute("formulas.evaluateFormula", json!({"formula": "={1.5,2.5;3,4}"})).unwrap();
    assert_eq!(r["resultLocal"], json!("{1,5.2,5;3.4}"));
    let r = s.execute("formulas.evaluateFormula", json!({"formula": "={\"a\",TRUE;#VALUE!,1}"})).unwrap();
    assert_eq!(r["resultLocal"], json!("{\"a\".WAHR;#WERT!.1}"));
    // An array that starts with an error is not blank.
    let r = s.execute("formulas.evaluateFormula", json!({"formula": "={#VALUE!,1}"})).unwrap();
    assert_eq!(r["resultLocal"], json!("{#WERT!.1}"));
}

#[test]
fn csv_import_is_canonical_unless_local() {
    let mut s = session_in(prefs("en-US", "pt-BR"));
    s.execute("data.fromTextCsv", json!({"text": "a,b\r\n1.5,2\r\n"})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(1.5));
    assert_eq!(v(&s, "B2"), Value::Number(2.0));
    s.execute("data.getData", json!({"text": "a,b\r\n2.5,3\r\n"})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(2.5));
    s.execute("data.fromTextCsv", json!({"text": "a;b\r\n1,5;2\r\n", "local": true})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(1.5));
    // Replaying the journal in en-US imports the same data, once per import.
    let mut replay = self::s();
    for (id, p) in s.journal.clone() {
        replay.execute(&id, p).unwrap();
    }
    assert_eq!(replay.doc().unwrap().wb.sheets.len(), s.doc().unwrap().wb.sheets.len());
    assert_eq!(v(&replay, "A2"), Value::Number(1.5));
}

#[test]
fn literal_validation_lists_are_quoted_and_canonical() {
    let mut s = session_in(prefs("en-US", "pt-BR"));
    s.execute("data.validation", json!({"range": "A1", "type": "list", "formula1Local": "Sim"})).unwrap();
    s.execute("data.validation", json!({"range": "A2", "type": "list", "formula1Local": "1,5;2"})).unwrap();
    let dv = |s: &Session, i: usize| s.doc().unwrap().wb.sheets[0].validations[i].f1.clone();
    assert_eq!(dv(&s, 0), "\"Sim\"");
    assert_eq!(dv(&s, 1), "\"1.5,2\"");
    assert_eq!(s.execute("data.validate", json!({"cell": "A1", "inputLocal": "Sim"})).unwrap()["ok"], true);
    assert_eq!(s.execute("data.validate", json!({"cell": "A1", "inputLocal": "Não"})).unwrap()["ok"], false);
    assert_eq!(s.execute("data.validate", json!({"cell": "A2", "inputLocal": "1,5"})).unwrap()["ok"], true);
    assert_eq!(s.execute("data.validate", json!({"cell": "A2", "inputLocal": "5"})).unwrap()["ok"], false);
    let mut replay = self::s();
    for (id, p) in s.journal.clone() {
        replay.execute(&id, p).unwrap();
    }
    assert_eq!(dv(&replay, 0), dv(&s, 0));
    assert_eq!(dv(&replay, 1), dv(&s, 1));
}

#[test]
fn define_name_reads_refers_to_in_the_call_locale() {
    let mut s = s();
    let r = s.execute("formulas.defineName", json!({"name": "X", "refersToLocal": "=SOMA(1,5;2)", "locale": "pt-BR"})).unwrap();
    assert_eq!(r["refersTo"], json!("=SUM(1.5,2)"));
    assert_eq!(r["refersToLocal"], json!("=SOMA(1,5;2)"));
}

#[test]
fn name_manager_spells_values_in_the_session_locale() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    s.execute("formulas.defineName", json!({"name": "Rate", "refersTo": "=1.5"})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Text", "refersTo": "=\"a\"\"b\""})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Flag", "refersTo": "=TRUE"})).unwrap();
    let names = s.execute("formulas.nameManager", json!({})).unwrap();
    let find = |n: &str| names.as_array().unwrap().iter().find(|x| x["name"] == n).unwrap().clone();
    assert_eq!(find("Rate")["valueLocal"], json!("1,5"));
    assert_eq!(find("Rate")["scope"], json!("Workbook"));
    assert_eq!(find("Flag")["valueLocal"], json!("WAHR"));
}
