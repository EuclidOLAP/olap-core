use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub calculator1); // synthesized by LALRPOP
lalrpop_mod!(pub calculator2);
lalrpop_mod!(pub calculator3);

#[test]
fn calculator1() {
    assert!(calculator2::TermParser::new().parse(">22@").is_ok());
    assert!(calculator2::TermParser::new().parse("(22)").is_ok());
    assert!(calculator2::TermParser::new().parse("((((22))))").is_ok());
    assert!(calculator2::TermParser::new().parse("((22)").is_err());
}

#[test]
fn calculator2() {
    assert!(calculator2::TermParser::new().parse(">22@").is_ok());
    assert!(calculator2::TermParser::new().parse("(22)").is_ok());
    assert!(calculator2::TermParser::new().parse("((((22))))").is_ok());
    assert!(calculator2::TermParser::new().parse("((22)").is_err());
}

#[test]
fn calculator3() {
    assert!(calculator3::ExprParser::new().parse("12-90+99").is_ok());
    assert!(calculator3::ExprParser::new().parse("12-90*99").is_ok());
    assert!(calculator3::ExprParser::new().parse("((22)+()").is_err());
}

fn main() {
    println!("Hello, world!");
}