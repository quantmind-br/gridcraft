//! German formulas: entry and display syntax.
use gridcraft_formula::FormulaLocale::{self, De};
use gridcraft_formula::{parse, parse_input, print, print_input};

fn canonical(src: &str, locale: FormulaLocale) -> String {
    print(&parse_input(src, locale, |_| false).unwrap_or_else(|e| panic!("{locale:?} `{src}`: {e}")))
}

fn shown(canonical: &str, locale: FormulaLocale) -> String {
    print_input(&parse(canonical).unwrap(), locale, |_| false).unwrap()
}

#[test]
fn german_formulas_read_and_show_like_german_excel() {
    for (local, english) in [
        ("SUMME(A1;2,5)", "SUM(A1,2.5)"),
        ("WENN(WAHR;SVERWEIS(A2;D:E;2;FALSCH);#NV)", "IF(TRUE,VLOOKUP(A2,D:E,2,FALSE),#N/A)"),
        ("SUMME({1.2;3,5.4})", "SUM({1,2;3.5,4})"),
        ("ZÄHLENWENN(Tabelle1[[#Kopfzeilen];[Spalte1]];\">1,5\")", "COUNTIF(Tabelle1[[#Headers],[Spalte1]],\">1,5\")"),
        ("WENNFEHLER(1/0;#WERT!)", "IFERROR(1/0,#VALUE!)"),
    ] {
        assert_eq!(canonical(local, De), english, "entry");
        assert_eq!(shown(english, De), local, "display");
    }
}

#[test]
fn english_names_any_letter_case_and_local_bindings() {
    assert!(De.translations().len() > 350);
    assert_eq!(canonical("SUM(1;2)+SUMME(3)", De), "SUM(1,2)+SUM(3)");
    assert_eq!(canonical("zählenwenn(A1:A3;1)", De), "COUNTIF(A1:A3,1)");
    assert_eq!(canonical("LET(zählenwenn;LAMBDA(x;x*2);ZÄHLENWENN(3))", De), "LET(zählenwenn,LAMBDA(x,x*2),ZÄHLENWENN(3))");
    assert_eq!(canonical("NACHZEILE(A1:B3;SUMME)", De), "BYROW(A1:B3,SUM)");
    let known = |name: &str| name == "SUMME";
    assert_eq!(print(&parse_input("NACHZEILE(A1:B3;SUMME)", De, known).unwrap()), "BYROW(A1:B3,SUMME)");
    assert_eq!(print_input(&parse("BYROW(A1:B3,SUM)").unwrap(), De, known).unwrap(), "NACHZEILE(A1:B3;SUM)");
    // English separators are not German syntax.
    assert!(parse_input("SUMME(1.5;2)", De, |_| false).is_err());
    for bad in ["SUMME({1.}", "Tabelle1[[#Kopfzeilen];", "#WERT"] {
        assert!(parse_input(bad, De, |_| false).is_err(), "{bad}");
    }
}
