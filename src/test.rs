use crate::grammar::ScriptParser;
use crate::lexer::Lexer;

#[test]
fn main() {
    let source_code = "var a = 42;
var b = 23;

# a comment
print (a - 1 + b);";

    let lexer = Lexer::new(source_code);
    let parser = ScriptParser::new();
    let ast = parser.parse(lexer).unwrap();

    println!("SSS---------------------------------------{:?}", ast);

    #[cfg(feature = "bit")]
    {
        let source_code = "var a = 4;
var b = 2;

# a comment
print (a << b);";

        let lexer = Lexer::new(source_code);
        let parser = ScriptParser::new();
        let ast = parser.parse(lexer).unwrap();

        println!("XXX---------------------------------------{:?}", ast);
    }
}

#[test]
fn test() {
    let source_code = "var a = 42;
var b = 23;
var xxx = b * b + 2 * a * b;
# a comment
print xxx * 1000 + (a - 1 + b);";

    let lexer = Lexer::new(source_code);
    let parser = ScriptParser::new();
    let ast = parser.parse(lexer).unwrap();

    println!("GGG---------------------------------------{:?}", ast);
}