use lalrpop_util::lalrpop_mod;

// lalrpop_mod!(pub calculator1); // synthesized by LALRPOP
lalrpop_mod!(pub calculator2);

#[test]
fn calculator1() {
    // assert!(calculator1::TermParser::new().parse("22").is_ok());
    // assert!(calculator1::TermParser::new().parse("(22)").is_ok());
    // assert!(calculator1::TermParser::new().parse("((((22))))").is_ok());
    // assert!(calculator1::TermParser::new().parse("((22)").is_err());
    assert!(calculator2::TermParser::new().parse(">22@").is_ok());
    assert!(calculator2::TermParser::new().parse("(22)").is_ok());
    assert!(calculator2::TermParser::new().parse("((((22))))").is_ok());
    assert!(calculator2::TermParser::new().parse("((22)").is_err());
    println!("calculator2 test passed .....................");
}

fn main() {
    println!("Hello, world!");
}