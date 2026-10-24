//! Russian formulas: entry and display syntax.
use gridcraft_formula::FormulaLocale::{self, Ru};
use gridcraft_formula::{parse, parse_input, print, print_input};

fn canonical(src: &str, locale: FormulaLocale) -> String {
    print(&parse_input(src, locale, |_| false).unwrap_or_else(|e| panic!("{locale:?} `{src}`: {e}")))
}

fn shown(canonical: &str, locale: FormulaLocale) -> String {
    print_input(&parse(canonical).unwrap(), locale, |_| false).unwrap()
}

#[test]
fn russian_formulas_read_and_show_like_russian_excel() {
    for (local, english) in [
        ("СУММ({1;2:3;4})", "SUM({1,2;3,4})"),
        ("СУММ(Таблица1[[#Итого];[Сумма]])", "SUM(Таблица1[[#Totals],[Сумма]])"),
        ("ЕСЛИ(ИСТИНА;СУММ(1,5;2);#Н/Д)", "IF(TRUE,SUM(1.5,2),#N/A)"),
    ] {
        assert_eq!(canonical(local, Ru), english, "entry");
        assert_eq!(shown(english, Ru), local, "display");
    }
}

#[test]
fn any_letter_case_colon_array_rows_and_local_bindings() {
    assert!(Ru.translations().len() > 350);
    assert_eq!(canonical("сумм(1;2)+#н/д", Ru), "SUM(1,2)+#N/A");
    // `:` separates array rows, so a number before it inside braces is not a row range.
    assert_eq!(canonical("{истина:ложь}+СУММ({1:2})+СУММ(1:3)", Ru), "{TRUE;FALSE}+SUM({1;2})+SUM(1:3)");
    // `,` is the decimal separator.
    assert!(parse_input("SUM(1,2)", Ru, |_| false).is_ok_and(|e| print(&e) == "SUM(1.2)"));
    let known = |name: &str| name == "СУММ";
    assert_eq!(print(&parse_input("СУММ(1)", Ru, known).unwrap()), "СУММ(1)");
    assert_eq!(print_input(&parse("SUM(1)").unwrap(), Ru, known).unwrap(), "SUM(1)");
    for bad in ["{ИСТИНА:", "СУММ(1:"] {
        assert!(parse_input(bad, Ru, |_| false).is_err(), "{bad}");
    }
}
