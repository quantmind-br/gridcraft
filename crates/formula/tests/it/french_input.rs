//! French formulas: entry and display syntax, checked against French Excel (#191, #443).
use gridcraft_formula::FormulaLocale::{self, Fr};
use gridcraft_formula::{parse, parse_input, print, print_input};

fn canonical(src: &str, locale: FormulaLocale) -> String {
    print(&parse_input(src, locale, |_| false).unwrap_or_else(|e| panic!("{locale:?} `{src}`: {e}")))
}

fn shown(canonical: &str, locale: FormulaLocale) -> String {
    print_input(&parse(canonical).unwrap(), locale, |_| false).unwrap()
}

#[test]
fn french_formulas_read_and_show_like_french_excel() {
    for (local, english) in [
        // Observed in French Excel 2013 (#191): decimal commas, `;` between arguments and rows.
        ("SOMME(1,5;2)+SOMME({1,2;3,4})", "SUM(1.5,2)+SUM({1.2;3.4})"),
        ("TEXTE(AUJOURDHUI();\"jj/mm/aaaa\")", "TEXT(TODAY(),\"jj/mm/aaaa\")"),
        ("LIGNES(Tableau1[#Tout])-LIGNES(Tableau1[[#En-têtes];[Nom]])", "ROWS(Tableau1[#All])-ROWS(Tableau1[[#Headers],[Nom]])"),
        ("SI(VRAI;RECHERCHEV(A2;D:E;2;FAUX);#N/A)", "IF(TRUE,VLOOKUP(A2,D:E,2,FALSE),#N/A)"),
        ("SOMME({1.2;3,5.4})+SIERREUR(1/0;#VALEUR!)", "SUM({1,2;3.5,4})+IFERROR(1/0,#VALUE!)"),
        // French Excel 2024 (#191, #443): STXTB exists, NBCARB/CHERCHEB/TROUVEB are #NOM?, and the
        // names below were retested there with that Excel's region separators.
        ("STXTB(A1;2;1)&CHERCHERB(\"b\";A1)&TROUVERB(\"b\";A1)&LENB(A1)", "MIDB(A1,2,1)&SEARCHB(\"b\",A1)&FINDB(\"b\",A1)&LENB(A1)"),
        (
            "SOMME(PRENDRE(A1:A6;2);EXCLURE(A1:A6;4);ETENDRE(A1:A2;3;1;0);DANSCOL(A1:A6);ORGA.COLS(A1:A6;3))",
            "SUM(TAKE(A1:A6,2),DROP(A1:A6,4),EXPAND(A1:A2,3,1,0),TOCOL(A1:A6),WRAPCOLS(A1:A6,3))",
        ),
        (
            "CENTILE.EXCLURE(A1:A6;0,5)+PREVISION.LINEAIRE(7;A1:A6;A1:A6)+SOMME(FILTRE(A1:A6;A1:A6>4))+RECHERCHEX(3;A1:A6;A1:A6)",
            "PERCENTILE.EXC(A1:A6,0.5)+FORECAST.LINEAR(7,A1:A6,A1:A6)+SUM(FILTER(A1:A6,A1:A6>4))+XLOOKUP(3,A1:A6,A1:A6)",
        ),
        ("URLENCODAGE(\"a b\")", "ENCODEURL(\"a b\")"),
        (
            "ET(ESTVIDE(Z1);ESTERR(1/0);ESTERREUR(1/0);ESTLOGIQUE(VRAI);ESTNA(#N/A);ESTNONTEXTE(1);NON(FAUX))",
            "AND(ISBLANK(Z1),ISERR(1/0),ISERROR(1/0),ISLOGICAL(TRUE),ISNA(#N/A),ISNONTEXT(1),NOT(FALSE))",
        ),
        ("ORGA.LIGNES(A1:A6;3)+BDLIRE(F1:G3;\"b\";I1:I2)", "WRAPROWS(A1:A6,3)+DGET(F1:G3,\"b\",I1:I2)"),
    ] {
        assert_eq!(canonical(local, Fr), english, "entry");
        assert_eq!(shown(english, Fr), local, "display");
    }
}

#[test]
fn local_names_that_are_other_english_names() {
    assert!(Fr.translations().len() > 350);
    // French TRIM is MIRR, and the English TRIM is SUPPRESPACE.
    assert_eq!(canonical("TRIM(A1:A3;0,1;0,1)&SUPPRESPACE(B1)", Fr), "MIRR(A1:A3,0.1,0.1)&TRIM(B1)");
    assert_eq!(shown("TRIM(MIRR(A1:A3,0.1,0.1))", Fr), "SUPPRESPACE(TRIM(A1:A3;0,1;0,1))");
}
