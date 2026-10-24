//! Brazilian Portuguese formulas: entry and display syntax, with the separators of a Brazilian
//! region.
use gridcraft_formula::FormulaLocale::{self, PtBr};
use gridcraft_formula::{Separators, parse, parse_input, print, print_input};

fn pt_br() -> Separators {
    Separators::new(PtBr, gridcraft_core::region::lookup("pt-BR"))
}

fn canonical(src: &str, locale: FormulaLocale) -> String {
    print(&parse_input(src, locale, pt_br(), |_| false).unwrap_or_else(|e| panic!("{locale:?} `{src}`: {e}")))
}

fn shown(canonical: &str, locale: FormulaLocale) -> String {
    print_input(&parse(canonical).unwrap(), locale, pt_br(), |_| false).unwrap()
}

#[test]
fn portuguese_formulas_read_and_show_like_brazilian_excel() {
    for (local, english) in [
        ("SE(VERDADEIRO;SOMA(1,5;2);#N/D)", "IF(TRUE,SUM(1.5,2),#N/A)"),
        ("PROCV(A2;Tabela1[#Tudo];2;FALSO)", "VLOOKUP(A2,Tabela1[#All],2,FALSE)"),
        ("SOMA({1\\2;3\\4})", "SUM({1,2;3,4})"),
    ] {
        assert_eq!(canonical(local, PtBr), english, "entry");
        assert_eq!(shown(english, PtBr), local, "display");
    }
}

#[test]
fn functions_passed_by_name_and_malformed_input() {
    assert!(PtBr.translations().len() > 350);
    assert_eq!(canonical("AGRUPARPOR(A2:A9;B2:B9;soma)", PtBr), "GROUPBY(A2:A9,B2:B9,SUM)");
    assert_eq!(shown("_xlfn.GROUPBY(A2:A9,B2:B9,_xleta.SUM)", PtBr), "AGRUPARPOR(A2:A9;B2:B9;SOMA)");
    for bad in ["SOMA(1,", "{SOMA(1)}"] {
        assert!(parse_input(bad, PtBr, pt_br(), |_| false).is_err(), "{bad}");
    }
}
