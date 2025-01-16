use crate::grammar::ScriptParser;
use crate::lexer::Lexer;

use crate::mdx_grammar::MdxStatementParser;
// use crate::mdx_grammar::BlocksChainParser;
use crate::mdx_grammar::SegParser;
use crate::mdx_grammar::SegmentsParser;
use crate::mdx_grammar::SegmentsWrapParser;
use crate::mdx_grammar::TupleWrapParser;
use crate::mdx_grammar::SetWrapParser;
use crate::mdx_grammar::AxisParser;
use crate::mdx_grammar::AxesParser;
use crate::mdx_grammar::SelectionMDXParser;

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

// Segments
#[test]
fn mdx_test_03_segments() {
    println!(">>> mdx_test_03_segments >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let ast_node = SegmentsParser::new().parse(MdxLexer::new("&100000123.&100000123.&100000123[>[[]]]]<]")).unwrap();
    println!(">>> mdx_test_03_segments {:?}", ast_node);
    let ast_node = SegmentsParser::new().parse(MdxLexer::new("[丰田].&100000123[兰德酷路泽].&100000123")).unwrap();
    println!(">>> mdx_test_03_segments {:?}", ast_node);
}

// SegmentsWrapParser
#[test]
fn mdx_test_04() {
    println!(">>> 04 SegmentsWrapParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let ast_node = SegmentsWrapParser::new().parse(MdxLexer::new("&100000123.&100000123.&100000123[>[[]]]]<]")).unwrap();
    println!(">>> 04 SegmentsWrapParser {:?}", ast_node);
}

// TupleWrapParser
#[test]
fn mdx_test_05() {
    println!("\n\n");

    println!(">>> 05 TupleWrapParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let ast_node = TupleWrapParser::new().parse(MdxLexer::new(
        "( [丰田].&100000123[兰德酷路泽].&100000123, [丰田].&100000123[兰德酷路泽], [丰田] )")).unwrap();
    println!("{:?}", ast_node);

    println!("\n\n");
}

// SetWrapParser
#[test]
fn mdx_test_06() {
    println!("\n\n");

    println!(">>> 06 SetWrapParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let mdx_fragment = "{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004692[企业客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
}";
    let ast_node = SetWrapParser::new().parse(MdxLexer::new(mdx_fragment)).unwrap();
    println!("{:?}", ast_node);

    println!("\n\n");
}

// AxisParser
#[test]
fn mdx_test_07() {
    println!("\n\n");
    println!(">>> 07 AxisParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

    let mdx_fragment = "{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
} on 090";
    let ast_node = AxisParser::new().parse(MdxLexer::new(mdx_fragment)).unwrap();
    println!("{:?}", ast_node);

    println!("\n\n");
}

// AxesParser
#[test]
fn mdx_test_08() {
    println!("\n\n");
    println!(">>> 08 AxesParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

    let mdx_fragment = "{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004692[企业客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
}
on 1,
{
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] )
}
on 0000";
    let ast_node = AxesParser::new().parse(MdxLexer::new(mdx_fragment)).unwrap();

    println!("{:?}", ast_node);
    println!("\n\n");
}

// SelectionMDXParser
#[test]
fn mdx_test_09() {
    println!("\n\n");
    println!(">>> 09 SelectionMDXParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

    let mdx_fragment = "select
{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004692[企业客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
}
on 1,
{
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] )
}
on 0
from &500000000000001[多维立方体]
where
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] )";
    let ast_node = SelectionMDXParser::new().parse(MdxLexer::new(mdx_fragment)).unwrap();

    println!("{:?}", ast_node);
    println!("\n\n");
}