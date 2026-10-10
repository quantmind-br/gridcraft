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
fn entering_empty_cells_preserves_whole_sheet_formatting() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "existing"})).unwrap();
    s.execute("selection.set", json!({"range": "A1:XFD1048576"})).unwrap();
    s.execute("home.alignCenter", json!({})).unwrap();
    let border = gridcraft_model::BorderLine { style: gridcraft_model::BorderStyle::Thin, color: gridcraft_model::Color::Auto };
    s.execute("home.formatCells", json!({"style": {"borders": {"top": border, "bottom": border, "left": border, "right": border}}})).unwrap();
    let expected = {
        let d = s.doc().unwrap();
        d.wb.styles.get(d.wb.active().unwrap().style_id(CellRef::parse("B1").unwrap())).clone()
    };
    assert_eq!(expected.align.h, gridcraft_model::HAlign::Center);
    for (cell, input) in [("A1", "edited"), ("B1", "new"), ("C1", "=1+2"), ("XFD1048576", "edge")] {
        s.execute("cell.set", json!({"cell": cell, "input": input})).unwrap();
        let d = s.doc().unwrap();
        assert_eq!(d.wb.styles.get(d.wb.active().unwrap().style_id(CellRef::parse(cell).unwrap())), &expected, "{cell}");
    }
    s.execute("range.setValues", json!({"range": "D1", "values": [["text", 7, true]]})).unwrap();
    for cell in ["D1", "E1", "F1"] {
        let d = s.doc().unwrap();
        assert_eq!(d.wb.styles.get(d.wb.active().unwrap().style_id(CellRef::parse(cell).unwrap())), &expected, "{cell}");
    }
    s.execute("edit.undo", json!({})).unwrap();
    assert!(v(&s, "D1").is_empty());
    s.execute("edit.redo", json!({})).unwrap();
    let d = s.doc().unwrap();
    assert_eq!(d.wb.styles.get(d.wb.active().unwrap().style_id(CellRef::parse("D1").unwrap())), &expected);
    assert_eq!(d.wb.active().unwrap().cells.len(), 7);
    let (reopened, _) = gridcraft_xlsx::read_xlsx(&gridcraft_xlsx::write_xlsx(&d.wb).unwrap()).unwrap();
    for cell in ["A1", "B1", "C1", "D1", "E1", "F1", "G1", "XFD1048576"] {
        assert_eq!(
            reopened.styles.get(reopened.active().unwrap().style_id(CellRef::parse(cell).unwrap())),
            &expected,
            "{cell} after XLSX round trip"
        );
    }
}

#[test]
fn entry_and_text_paste_use_inherited_formats_and_cell_overrides() {
    let mut s = s();
    s.execute("home.bold", json!({"range": "B:B", "on": true})).unwrap();
    s.execute("home.formatCells", json!({"range": "3:3", "style": {"num_fmt": "@", "font": {"italic": true}}})).unwrap();
    s.execute("home.fontColor", json!({"range": "B3", "color": "#FF0000"})).unwrap();
    let expected = {
        let d = s.doc().unwrap();
        d.wb.styles.get(d.wb.active().unwrap().style_id(CellRef::parse("B3").unwrap())).clone()
    };
    s.execute("cell.set", json!({"cell": "B3", "input": "=1+2"})).unwrap();
    s.execute("cell.set", json!({"cell": "C3", "input": "00123"})).unwrap();
    s.execute("edit.paste", json!({"at": "D3", "text": "=2+3\t00456"})).unwrap();
    assert_eq!(v(&s, "B3"), Value::text("=1+2"));
    assert_eq!(v(&s, "C3"), Value::text("00123"));
    assert_eq!(v(&s, "D3"), Value::text("=2+3"));
    assert_eq!(v(&s, "E3"), Value::text("00456"));
    s.execute("cell.set", json!({"cell": "B4", "input": "12%"})).unwrap();
    let d = s.doc().unwrap();
    let sh = d.wb.active().unwrap();
    assert_eq!(d.wb.styles.get(sh.style_id(CellRef::parse("B3").unwrap())), &expected);
    for cell in ["C3", "D3", "E3"] {
        let st = d.wb.styles.get(sh.style_id(CellRef::parse(cell).unwrap()));
        assert!(st.font.italic);
        assert!(!st.font.bold, "row style takes precedence over column style");
        assert_eq!(st.num_fmt.as_str(), "@");
    }
    let percent = d.wb.styles.get(sh.style_id(CellRef::parse("B4").unwrap()));
    assert!(percent.font.bold);
    assert_ne!(percent.num_fmt.as_str(), "General");
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
fn no_border_clears_only_shared_edges_and_undo_restores_them() {
    use gridcraft_core::RangeRef;
    use gridcraft_model::{BorderLine, Borders};

    for (range, neighbors) in [
        ("B2", &[("B1", "bottom"), ("B3", "top"), ("A2", "right"), ("C2", "left")][..]),
        (
            "B2:C3",
            &[("B1", "bottom"), ("C1", "bottom"), ("B4", "top"), ("C4", "top"), ("A2", "right"), ("A3", "right"), ("D2", "left"), ("D3", "left")][..],
        ),
    ] {
        let mut s = s();
        s.execute("range.setValues", json!({"range": "A1", "values": [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12], [13, 14, 15, 16]]})).unwrap();
        s.execute("cell.set", json!({"cell": "D4", "input": "=SUM(A1:C1)"})).unwrap();
        s.execute("home.bold", json!({"range": "A1:D4", "on": true})).unwrap();
        for preset in ["all", "diagonalDown"] {
            s.execute("home.borders", json!({"range": "A1:D4", "preset": preset, "style": "double", "color": "#CC3344"})).unwrap();
        }
        let before: Vec<_> = RangeRef::parse("A1:D4").unwrap().iter().map(|c| s.execute("cell.get", json!({"cell": c.a1()})).unwrap()).collect();
        s.execute("home.borders", json!({"range": range, "preset": "none"})).unwrap();
        let selected = RangeRef::parse(range).unwrap();
        for original in &before {
            let address = original["cell"].as_str().unwrap();
            let mut expected = original.clone();
            if selected.contains(CellRef::parse(address).unwrap()) {
                expected["style"]["borders"] = json!(Borders::default());
            } else if let Some((_, edge)) = neighbors.iter().find(|(cell, _)| *cell == address) {
                expected["style"]["borders"][*edge] = json!(BorderLine::default());
            }
            assert_eq!(s.execute("cell.get", json!({"cell": address})).unwrap(), expected, "clearing {range}, cell {address}");
        }
        s.execute("edit.undo", json!({})).unwrap();
        for original in before {
            assert_eq!(s.execute("cell.get", json!({"cell": original["cell"]})).unwrap(), original);
        }
    }
}

#[test]
fn no_border_handles_sheet_corners_without_creating_out_of_bounds_cells() {
    for (range, corner, neighbors) in [
        ("A1:B2", "A1", [("B1", "left"), ("A2", "top")]),
        ("XFC1048575:XFD1048576", "XFD1048576", [("XFC1048576", "right"), ("XFD1048575", "bottom")]),
    ] {
        let mut s = s();
        s.execute("home.borders", json!({"range": range, "preset": "all"})).unwrap();
        s.execute("home.borders", json!({"range": corner, "preset": "none"})).unwrap();
        for (neighbor, edge) in neighbors {
            assert_eq!(s.execute("cell.get", json!({"cell": neighbor})).unwrap()["style"]["borders"][edge]["style"], "None");
        }
        let sh = s.doc().unwrap().wb.active().unwrap();
        assert!(sh.cells.iter().all(|(c, _)| c.is_valid()));
        assert!(sh.cells.iter().count() <= 4);
    }
}

#[test]
fn borders_respect_sheet_formatting_protection() {
    let mut s = s();
    s.execute("home.borders", json!({"range": "A1:B2", "preset": "all"})).unwrap();
    s.execute("review.protectSheet", json!({})).unwrap();
    let before = s.doc().unwrap().wb.clone();
    let undo_count = s.doc().unwrap().undo.len();
    for preset in ["none", "left"] {
        assert!(s.execute("home.borders", json!({"range": "A1", "preset": preset})).is_err());
        assert_eq!(s.doc().unwrap().wb.styles, before.styles);
        assert_eq!(s.doc().unwrap().wb.active().unwrap().cells, before.active().unwrap().cells);
        assert_eq!(s.doc().unwrap().undo.len(), undo_count);
    }
    s.execute("review.protectSheet", json!({"formatCells": true})).unwrap();
    s.execute("home.borders", json!({"range": "A1", "preset": "none"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "B1"})).unwrap()["style"]["borders"]["left"]["style"], "None");
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
fn inserting_or_deleting_cells_cancels_copy_mode() {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1], [2], [3]]})).unwrap();
    s.execute("edit.cut", json!({"range": "3:3"})).unwrap();
    s.execute("home.insertRows", json!({"rows": "3:3"})).unwrap();
    assert!(s.clipboard.is_none());
    // The cut data is now in row 4; a paste must not move the new blank row 3 instead.
    assert!(s.execute("edit.paste", json!({"at": "A10"})).is_err());
    assert_eq!(v(&s, "A4"), Value::Number(3.0));
    s.execute("edit.copy", json!({"range": "A:A"})).unwrap();
    s.execute("home.deleteColumns", json!({"cols": "B:B"})).unwrap();
    assert!(s.clipboard.is_none());
    s.execute("edit.copy", json!({"range": "A1"})).unwrap();
    s.execute("home.insertCells", json!({"range": "A1", "shift": "down"})).unwrap();
    assert!(s.clipboard.is_none());
    s.execute("edit.copy", json!({"range": "A2"})).unwrap();
    s.execute("home.deleteCells", json!({"range": "A1", "shift": "up"})).unwrap();
    assert!(s.clipboard.is_none());
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
fn insert_delete_cells_adjust() {
    let formula = |s: &mut Session, a: &str| s.execute("cell.get", json!({"cell": a})).unwrap()["formula"].clone();
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1], [2], [3], [4], [5]]})).unwrap();
    s.execute("cell.set", json!({"cell": "C1", "input": "=SUM(A1:A5)"})).unwrap();
    s.execute("cell.set", json!({"cell": "C2", "input": "=A4*10"})).unwrap();
    s.execute("cell.set", json!({"cell": "C3", "input": "=SUM(A1:B5)"})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Fourth", "refersTo": "=Sheet1!$A$4"})).unwrap();
    s.execute(
        "home.conditionalFormat",
        json!({"range": "A4:A5", "rule": {"type": "cellIs", "operator": "greater", "value": "3", "preset": "redText"}}),
    )
    .unwrap();
    s.execute("data.validation", json!({"range": "A5", "type": "whole", "operator": "greater", "formula1": "0"})).unwrap();
    s.execute("insert.chart", json!({"range": "A1:A5", "type": "line"})).unwrap();
    // Insert A3 shifting down: ranges across the insertion grow, references below move.
    s.execute("home.insertCells", json!({"range": "A3", "shift": "down"})).unwrap();
    assert_eq!(formula(&mut s, "C1"), "=SUM(A1:A6)");
    assert_eq!(formula(&mut s, "C2"), "=A5*10");
    assert_eq!(formula(&mut s, "C3"), "=SUM(A1:B5)");
    assert_eq!(v(&s, "C2"), Value::Number(40.0));
    let sh = s.doc().unwrap().wb.active().unwrap().clone();
    assert_eq!(s.doc().unwrap().wb.names[0].formula, "Sheet1!$A$5");
    assert_eq!(sh.cond_formats[0].ranges[0].a1(), "A5:A6");
    assert_eq!(sh.validations[0].ranges[0].a1(), "A6");
    assert_eq!(sh.charts[0].series[0].values, "Sheet1!$A$1:$A$6");
    // Delete A5 (the 4) shifting up: references to it become #REF!, those below move up.
    s.execute("home.deleteCells", json!({"range": "A5", "shift": "up"})).unwrap();
    assert_eq!(formula(&mut s, "C1"), "=SUM(A1:A5)");
    assert_eq!(formula(&mut s, "C2"), "=#REF!*10");
    assert_eq!(s.doc().unwrap().wb.names[0].formula, "#REF!");
    assert_eq!(v(&s, "C1"), Value::Number(11.0));
    let sh = s.doc().unwrap().wb.active().unwrap().clone();
    assert_eq!(sh.cond_formats[0].ranges[0].a1(), "A5");
    assert_eq!(sh.validations[0].ranges[0].a1(), "A5");
    assert_eq!(sh.charts[0].series[0].values, "Sheet1!$A$1:$A$5");
    // Shifting right and left.
    s.execute("range.setValues", json!({"range": "E1", "values": [[1, 2, 3]]})).unwrap();
    s.execute("cell.set", json!({"cell": "E3", "input": "=G1+SUM(E1:G1)"})).unwrap();
    s.execute("home.insertCells", json!({"range": "F1", "shift": "right"})).unwrap();
    assert_eq!(formula(&mut s, "E3"), "=H1+SUM(E1:H1)");
    s.execute("home.deleteCells", json!({"range": "E1:F1", "shift": "left"})).unwrap();
    assert_eq!(formula(&mut s, "E3"), "=F1+SUM(E1:F1)");
    assert_eq!(v(&s, "E3"), Value::Number(8.0));
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

/// A sheet with locked data in A1:B3, unlocked cells in D1:D3 and row 10, then protected.
fn protected(extra: serde_json::Value) -> Session {
    let mut s = s();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1], [2], [3]]})).unwrap();
    s.execute("cell.set", json!({"cell": "B1", "input": "=A1+D1"})).unwrap();
    s.execute("range.setValues", json!({"range": "D1", "values": [[10], [20], [30]]})).unwrap();
    s.execute("home.lockCell", json!({"range": "D1:D3", "on": false})).unwrap();
    s.execute("home.lockCell", json!({"range": "10:10", "on": false})).unwrap();
    s.execute("review.protectSheet", extra).unwrap();
    s
}

#[test]
fn protection_refuses_edits_of_locked_cells() {
    let refused = |s: &mut Session, id: &str, p: serde_json::Value| {
        let before = s.doc().unwrap().wb.clone();
        let e = s.execute(id, p).expect_err(id).to_string();
        assert!(e.contains("protected sheet"), "{id}: {e}");
        assert_eq!(*s.doc().unwrap().wb, *before, "{id} changed the workbook");
    };
    let mut s = protected(json!({}));
    refused(&mut s, "cell.set", json!({"cell": "A1", "input": "9"}));
    refused(&mut s, "edit.clearContents", json!({"range": "A1:A3"}));
    refused(&mut s, "edit.clearAll", json!({"range": "A1"}));
    refused(&mut s, "range.setValues", json!({"range": "A2", "values": [[9]]}));
    refused(&mut s, "range.fill", json!({"range": "A5:A6", "input": "x"}));
    refused(&mut s, "edit.fillDown", json!({"range": "A1:A3"}));
    refused(&mut s, "edit.fillRight", json!({"range": "A1:B1"}));
    refused(&mut s, "edit.fillUp", json!({"range": "A1:A3"}));
    refused(&mut s, "edit.fillLeft", json!({"range": "A1:B1"}));
    refused(&mut s, "edit.autoFill", json!({"source": "A1", "target": "A1:A5"}));
    s.execute("edit.copy", json!({"range": "D1"})).unwrap();
    refused(&mut s, "edit.paste", json!({"at": "A2"}));
    refused(&mut s, "edit.paste", json!({"at": "A2", "text": "x\ty"}));
    refused(&mut s, "home.deleteRows", json!({"rows": "2:2"}));
    refused(&mut s, "home.deleteColumns", json!({"cols": "A:A"}));
    refused(&mut s, "home.insertRows", json!({"rows": "1:1"}));
    refused(&mut s, "home.deleteCells", json!({"range": "A1", "shift": "up"}));
    refused(&mut s, "home.insertCells", json!({"range": "A1", "shift": "down"}));
    refused(&mut s, "edit.replace", json!({"what": "2", "with": "5"}));
    refused(&mut s, "data.goalSeek", json!({"set": "B1", "to": 50, "changing": "A1"}));
    // Unlocked cells stay editable.
    s.execute("edit.paste", json!({"at": "D2"})).unwrap();
    assert_eq!(v(&s, "D2"), Value::Number(10.0));
    s.execute("range.setValues", json!({"range": "D2", "values": [[5]]})).unwrap();
    s.execute("edit.fillDown", json!({"range": "D1:D3"})).unwrap();
    assert_eq!(v(&s, "D3"), Value::Number(10.0));
    s.execute("edit.clearContents", json!({"range": "D3"})).unwrap();
    assert_eq!(v(&s, "D3"), Value::Empty);
    s.execute("data.goalSeek", json!({"set": "B1", "to": 50, "changing": "D1"})).unwrap();
    assert!((v(&s, "B1").as_f64().unwrap() - 50.0).abs() < 0.001);
    s.execute("edit.replace", json!({"what": "49", "with": "48", "wholeCell": true})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(1.0));
}

#[test]
fn protection_allows_permitted_row_and_column_edits() {
    let mut s = protected(json!({"insertRows": true, "deleteRows": true, "insertColumns": true}));
    // Inserting shifts locked cells, which the protection allows.
    s.execute("home.insertRows", json!({"rows": "1:1"})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(1.0));
    s.execute("home.insertColumns", json!({"cols": "A:A"})).unwrap();
    // Deleting a row with locked cells isn't; deleting an unlocked row is.
    assert!(s.execute("home.deleteRows", json!({"rows": "2:2"})).is_err());
    s.execute("home.deleteRows", json!({"rows": "11:11"})).unwrap();
    assert!(s.execute("home.deleteColumns", json!({"cols": "Z:Z"})).is_err());
}

#[test]
fn hidden_formulas_stay_hidden_on_protected_sheets() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "=SEQUENCE(3)*7"})).unwrap();
    s.execute("cell.set", json!({"cell": "B1", "input": "=1+1"})).unwrap();
    s.execute("home.formatCells", json!({"range": "A1", "style": {"protection": {"locked": true, "hidden": true}}})).unwrap();
    s.execute("formulas.showFormulas", json!({"on": true})).unwrap();
    let text = |s: &Session, a: &str| {
        let d = s.doc().unwrap();
        let sh = d.wb.active().unwrap();
        crate::display::cell_text(&d.wb, sh, CellRef::parse(a).unwrap())
    };
    assert_eq!(text(&s, "A1"), "=SEQUENCE(3)*7");
    assert_eq!(s.execute("edit.find", json!({"what": "SEQUENCE", "all": true})).unwrap()["count"], 1);
    s.execute("review.protectSheet", json!({})).unwrap();
    assert_eq!(text(&s, "A1"), "7");
    assert_eq!(text(&s, "B1"), "=1+1");
    let d = s.doc().unwrap();
    let sh = d.wb.active().unwrap();
    assert!(crate::display::formula_hidden(&d.wb, sh, CellRef::parse("A2").unwrap()));
    assert!(!crate::display::formula_hidden(&d.wb, sh, CellRef::parse("B1").unwrap()));
    assert!(s.execute("edit.find", json!({"what": "SEQUENCE", "all": true})).is_err());
    let r = s.execute("edit.find", json!({"what": "7", "all": true})).unwrap();
    assert_eq!(r["results"][0]["value"], "7");
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
fn spill_references_roundtrip_through_xlsx() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "B1", "input": "=SEQUENCE(4)"})).unwrap();
    s.execute("cell.set", json!({"cell": "C1", "input": "=sum(b1#)"})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Seq", "refersTo": "=Sheet1!$B$1#"})).unwrap();
    s.execute("cell.set", json!({"cell": "C2", "input": "=MAX(Seq)"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "C1"})).unwrap()["formula"], "=SUM(B1#)");
    assert_eq!((v(&s, "C1"), v(&s, "C2")), (Value::Number(10.0), Value::Number(4.0)));
    let r = s.execute("file.saveBytes", json!({"format": "xlsx"})).unwrap();
    let b64 = r["base64"].as_str().unwrap().to_string();
    s.execute("file.open", json!({"name": "x.xlsx", "base64": b64})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "C1"})).unwrap()["formula"], "=SUM(B1#)");
    assert_eq!(s.doc().unwrap().wb.names[0].formula, "Sheet1!$B$1#");
    assert_eq!((v(&s, "C1"), v(&s, "C2")), (Value::Number(10.0), Value::Number(4.0)));
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
fn create_names_from_selection() {
    let names = |s: &Session| -> Vec<(String, String)> {
        let mut v: Vec<_> = s.doc().unwrap().wb.names.iter().map(|n| (n.name.clone(), n.formula.clone())).collect();
        v.sort();
        v
    };
    let named = |n: &str, f: &str| (n.to_string(), f.to_string());
    let mut s = s();
    s.execute(
        "range.setValues",
        json!({"range": "A1", "values": [["", "Jan", "Feb", ""], ["North", 1, 2, "N"], ["South", 3, 4, "S"], ["", "First", "Second", ""]]}),
    )
    .unwrap();
    s.execute("formulas.createFromSelection", json!({"range": "A1:D4", "top": false, "bottom": true, "right": true})).unwrap();
    assert_eq!(
        names(&s),
        [named("First", "Sheet1!$B$1:$B$3"), named("N", "Sheet1!$A$2:$C$2"), named("S", "Sheet1!$A$3:$C$3"), named("Second", "Sheet1!$C$1:$C$3")]
    );
    // Existing names are kept unless replacing is asked for.
    let r = s.execute("formulas.createFromSelection", json!({"range": "A1:C3", "top": true, "left": true})).unwrap();
    assert_eq!(r["created"], 4);
    let r = s.execute("formulas.createFromSelection", json!({"range": "B1:C3", "top": true})).unwrap();
    assert_eq!((r["created"].clone(), r["skipped"].clone()), (json!(0), json!(["Jan", "Feb"])));
    s.execute("cell.set", json!({"cell": "B1", "input": "N"})).unwrap();
    let r = s.execute("formulas.createFromSelection", json!({"range": "B1:B3", "replace": true})).unwrap();
    assert_eq!(r["created"], 1);
    assert!(names(&s).contains(&named("N", "Sheet1!$B$2:$B$3")));
    assert!(names(&s).contains(&named("North", "Sheet1!$B$2:$C$2")));
}

#[test]
fn names_that_look_like_references_are_refused() {
    let mut s = s();
    for bad in ["R1C1", "r2", "C3", "RC", "R", "c", "rc12", "A1", "XFD1048576", "1st"] {
        assert!(s.execute("formulas.defineName", json!({"name": bad, "refersTo": "=1"})).is_err(), "{bad}");
    }
    for good in ["R1C1X", "Rate", "RCx", "Rx", "_R1", "Cost"] {
        s.execute("formulas.defineName", json!({"name": good, "refersTo": "=1"})).unwrap();
    }
}

#[test]
fn lambda_names_keep_their_case_in_calls() {
    let mut s = s();
    s.execute("formulas.defineName", json!({"name": "Double", "refersTo": "=LAMBDA(x,x*2)"})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Quad", "refersTo": "=LAMBDA(x,Double(Double(x)))"})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "input": "=double(4)+Quad(1)+sum(1)"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["formula"], "=Double(4)+Quad(1)+SUM(1)");
    assert_eq!(v(&s, "A1"), Value::Number(13.0));
    let r = s.execute("file.saveBytes", json!({"format": "xlsx"})).unwrap();
    let b64 = r["base64"].as_str().unwrap().to_string();
    s.execute("file.open", json!({"name": "x.xlsx", "base64": b64})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["formula"], "=Double(4)+Quad(1)+SUM(1)");
    let quad = s.doc().unwrap().wb.names.iter().find(|n| n.name == "Quad").unwrap().formula.clone();
    assert_eq!(quad, "LAMBDA(x,Double(Double(x)))");
    assert_eq!(v(&s, "A1"), Value::Number(13.0));
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
fn sheet_operations_keep_names_on_their_sheets() {
    let names = |s: &Session| -> Vec<(String, Option<String>, String)> {
        let wb = &s.doc().unwrap().wb;
        let mut v: Vec<_> =
            wb.names.iter().map(|n| (n.name.clone(), n.scope.and_then(|i| wb.sheet(i)).map(|s| s.name.clone()), n.formula.clone())).collect();
        v.sort();
        v
    };
    let named = |n: &str, scope: Option<&str>, f: &str| (n.to_string(), scope.map(str::to_string), f.to_string());
    let mut s = s();
    s.execute("home.insertSheet", json!({"name": "Data"})).unwrap();
    s.execute("home.insertSheet", json!({"name": "Notes"})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Rate", "refersTo": "=Data!$A$1", "scope": "Data"})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Rate", "refersTo": "=Notes!$B$1", "scope": "Notes"})).unwrap();
    s.execute("formulas.defineName", json!({"name": "Total", "refersTo": "=Data!$C$1"})).unwrap();
    // Moving Data (index 1) to the end: its names go with it.
    s.execute("sheet.move", json!({"sheet": "Data", "to": 2})).unwrap();
    assert_eq!(names(&s), [named("Rate", Some("Data"), "Data!$A$1"), named("Rate", Some("Notes"), "Notes!$B$1"), named("Total", None, "Data!$C$1")]);
    // Copying Data to the front copies its sheet-level names, referring to the copy.
    s.execute("sheet.move", json!({"sheet": "Data", "to": 0, "copy": true})).unwrap();
    assert_eq!(s.doc().unwrap().wb.sheets[0].name, "Data (2)");
    assert_eq!(
        names(&s),
        [
            named("Rate", Some("Data"), "Data!$A$1"),
            named("Rate", Some("Data (2)"), "'Data (2)'!$A$1"),
            named("Rate", Some("Notes"), "Notes!$B$1"),
            named("Total", None, "Data!$C$1")
        ]
    );
    // Deleting Data removes its names; names referring to it become #REF!.
    s.execute("home.deleteSheet", json!({"sheet": "Data"})).unwrap();
    assert_eq!(
        names(&s),
        [named("Rate", Some("Data (2)"), "'Data (2)'!$A$1"), named("Rate", Some("Notes"), "Notes!$B$1"), named("Total", None, "#REF!")]
    );
    // Deleting a name in one scope keeps the others.
    s.execute("formulas.deleteName", json!({"name": "rate", "scope": "Notes"})).unwrap();
    assert_eq!(names(&s), [named("Rate", Some("Data (2)"), "'Data (2)'!$A$1"), named("Total", None, "#REF!")]);
    assert!(s.execute("formulas.deleteName", json!({"name": "Total", "scope": "Notes"})).is_err());
    s.execute("formulas.deleteName", json!({"name": "Total", "scope": "Workbook"})).unwrap();
    assert_eq!(names(&s).len(), 1);
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
    assert_eq!(E::Protected.code(), "protected-sheet");
    assert!(E::Protected.args().is_empty());
    assert!(E::Protected.to_string().contains("protected sheet"));
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
fn hidden_formulas_stay_hidden_in_every_spelling() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    s.execute("cell.set", json!({"cell": "A1", "inputLocal": "=SUMME(1,5;2)+\"x\""})).unwrap();
    s.execute("cell.set", json!({"cell": "B1", "inputLocal": "=SUMME(1,5;3)"})).unwrap();
    s.execute("home.formatCells", json!({"range": "A1", "style": {"protection": {"locked": true, "hidden": true}}})).unwrap();
    s.execute("formulas.showFormulas", json!({"on": true})).unwrap();
    s.execute("review.protectSheet", json!({})).unwrap();
    let leaks = |v: &serde_json::Value| v.to_string().contains("SUM");
    // Neither spelling of the formula, nor the input that would reveal it.
    let a1 = s.execute("cell.get", json!({"cell": "A1"})).unwrap();
    assert_eq!(a1["formula"], serde_json::Value::Null);
    assert_eq!(a1["formulaLocal"], serde_json::Value::Null);
    assert!(!leaks(&a1), "{a1}");
    assert_eq!(a1["inputLocal"], "#WERT!");
    // A formula that is not hidden still shows both.
    let b1 = s.execute("cell.get", json!({"cell": "B1"})).unwrap();
    assert_eq!(b1["formula"], "=SUM(1.5,3)");
    assert_eq!(b1["formulaLocal"], "=SUMME(1,5;3)");
    for local in [false, true] {
        let r = s.execute("sheet.read", json!({"range": "A1", "formulas": true, "local": local})).unwrap();
        assert!(!leaks(&r), "{r}");
    }
    let r = s.execute("sheet.read", json!({"range": "B1", "formulas": true, "local": true})).unwrap();
    assert_eq!(r["values"][0][0], "=SUMME(1,5;3)");
    let w = s.execute("formulas.watchWindow", json!({"add": "A1"})).unwrap();
    assert_eq!((&w[0]["formula"], &w[0]["formulaLocal"]), (&serde_json::Value::Null, &serde_json::Value::Null));
    let report = s.execute("formulas.errorChecking", json!({})).unwrap();
    let row = &report["errors"][0];
    assert_eq!(row["cell"], "A1");
    assert_eq!((&row["formula"], &row["formulaLocal"]), (&serde_json::Value::Null, &serde_json::Value::Null));
    assert_eq!(row["errorLocal"], "#WERT!");
    assert_eq!(s.execute("formulas.evaluateFormula", json!({"cell": "A1"})).unwrap_err().code(), "protected-sheet");
    // The formula can't be found or replaced by its text either.
    assert!(s.execute("edit.find", json!({"what": "SUMME(1,5;2)", "all": true})).is_err());
    assert_eq!(s.execute("edit.replace", json!({"what": "SUMME(1,5;2)", "with": "X"})).unwrap()["replaced"], 0);
    let d = s.doc().unwrap();
    let sh = d.wb.active().unwrap();
    // With Show Formulas on, only the formula that isn't hidden is shown.
    let shown = crate::display::cell_text(&d.wb, sh, CellRef::parse("A1").unwrap());
    assert!(!shown.contains("SUM") && !shown.starts_with('='), "{shown}");
    assert_eq!(crate::display::cell_text(&d.wb, sh, CellRef::parse("B1").unwrap()), "=SUMME(1,5;3)");
}

#[test]
fn input_local_obeys_sheet_protection_like_input() {
    let mut s = session_in(prefs("de-DE", "de-DE"));
    s.execute("cell.set", json!({"cell": "A1", "input": "1"})).unwrap();
    s.execute("range.setValues", json!({"range": "D1", "values": [[1], [2], [3]]})).unwrap();
    s.execute("home.lockCell", json!({"range": "D1:D3", "on": false})).unwrap();
    s.execute("review.protectSheet", json!({})).unwrap();
    let before = s.doc().unwrap().wb.clone();
    for (id, p) in [
        ("cell.set", json!({"cell": "A1", "inputLocal": "=SUMME(1,5;2)"})),
        ("cell.set", json!({"cell": "A1", "input": "=SUM(1.5,2)"})),
        ("range.fill", json!({"range": "A1:A2", "inputLocal": "1,5"})),
        ("range.setValues", json!({"range": "A1", "values": [["1,5"]], "local": true})),
    ] {
        let e = s.execute(id, p).expect_err(id);
        assert_eq!(e.code(), "protected-sheet", "{id}");
        assert_eq!(*s.doc().unwrap().wb, *before, "{id} changed the workbook");
    }
    // Unlocked cells still take local input.
    s.execute("cell.set", json!({"cell": "D1", "inputLocal": "1,5"})).unwrap();
    assert_eq!(v(&s, "D1"), Value::Number(1.5));
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
