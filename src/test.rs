use crate::grammar::ScriptParser;
use crate::lexer::Lexer;

use crate::mdx_grammar::MdxStatementParser;
// use crate::mdx_grammar::BlocksChainParser;
use crate::mdx_grammar::SegParser;
use crate::mdx_lexer::Lexer as MdxLexer;

#[test]
fn main() {
    let source_code = "var a = 42;
Var b = 23;

# a comment
  -- other one comment
prINT (a - 1 + b);
-- other one comment";

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

#[test]
fn mdx_test_01() {
    let source_code = "Select --{}\n on from where";

    let lexer = MdxLexer::new(source_code);
    let parser = MdxStatementParser::new();
    let ast = parser.parse(lexer).unwrap();

    println!("MDX---------------------------------------{:?}", ast);
}

#[test]
fn mdx_test_02() {
    let ast_node = SegParser::new().parse(MdxLexer::new("&0000000000000000000000000300000000321")).unwrap();
    println!("MDX TEST 02 >>>>>>>>>>>>>>>>>>>>> {:?}", ast_node);
    let ast_node = SegParser::new().parse(MdxLexer::new("&0000000000000000000000000000000000000000000000000000000000000000000300000000345[LA]")).unwrap();
    println!("MDX TEST 02 >>>>>>>>>>>>>>>>>>>>> {:?}", ast_node);
    let ast_node = SegParser::new().parse(MdxLexer::new("[[iPhone 16]] pro max]")).unwrap();
    println!("MDX TEST 02 >>>>>>>>>>>>>>>>>>>>> {:?}", ast_node);
}