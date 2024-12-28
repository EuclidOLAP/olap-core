use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    character::complete::digit1,
    // character::complete::{alpha1, space0},
    combinator::map,
    multi::many1,
    sequence::delimited,
    IResult,
};

// Token 枚举类型
#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    // Number(i64),
    // Identifier(String),
    Whitespace,   // 空白符
    Semicolon,    // ;
    Comma,        // ,
    ParenOpen,    // (
    ParenClose,   // )
    BraceOpen,    // {
    BraceClose,   // }
    Dot,          // .
    Ampersand,    // &
    BracketOpen,  // [
    BracketClose, // ]
    UnsignedLong(u64),
    BracketString(String),
}

fn keyword(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag_no_case("SELECT"), |_| {
            Token::Keyword("SELECT".to_string())
        }),
        map(tag_no_case("FROM"), |_| Token::Keyword("FROM".to_string())),
        map(tag_no_case("WHERE"), |_| {
            Token::Keyword("WHERE".to_string())
        }),
        map(tag_no_case("ON"), |_| Token::Keyword("ON".to_string())),
        map(tag_no_case("ROWS"), |_| Token::Keyword("ROWS".to_string())),
        map(tag_no_case("COLUMNS"), |_| {
            Token::Keyword("COLUMNS".to_string())
        }),
    ))(input)
}

fn whitespace(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag(" "), |_| Token::Whitespace),
        map(tag("\n"), |_| Token::Whitespace),
        map(tag("\t"), |_| Token::Whitespace),
    ))(input)
}

fn semicolon(input: &str) -> IResult<&str, Token> {
    map(tag(";"), |_| Token::Semicolon)(input)
}

// 词法解析函数：解析逗号
fn comma(input: &str) -> IResult<&str, Token> {
    // tag(",")：精确匹配逗号字符 ,
    // map：匹配成功后返回 Token::Comma
    map(tag(","), |_| Token::Comma)(input)
}

// 词法解析函数：解析括号
// 解析括号（paren_open 和 paren_close）
// tag("(") 和 tag(")")：分别匹配左括号和右括号。
// map：匹配成功后分别返回 Token::ParenOpen 和 Token::ParenClose。
fn paren_open(input: &str) -> IResult<&str, Token> {
    map(tag("("), |_| Token::ParenOpen)(input)
}

fn paren_close(input: &str) -> IResult<&str, Token> {
    map(tag(")"), |_| Token::ParenClose)(input)
}

// 词法解析函数：解析花括号
fn brace_open(input: &str) -> IResult<&str, Token> {
    map(tag("{"), |_| Token::BraceOpen)(input)
}

fn brace_close(input: &str) -> IResult<&str, Token> {
    map(tag("}"), |_| Token::BraceClose)(input)
}

// 词法解析函数：解析点
fn dot(input: &str) -> IResult<&str, Token> {
    map(tag("."), |_| Token::Dot)(input)
}

// 解析 "&"
fn ampersand(input: &str) -> IResult<&str, Token> {
    map(tag("&"), |_| Token::Ampersand)(input)
}

// 解析 "["
fn bracket_open(input: &str) -> IResult<&str, Token> {
    map(tag("["), |_| Token::BracketOpen)(input)
}

// 解析 "]"
fn bracket_close(input: &str) -> IResult<&str, Token> {
    map(tag("]"), |_| Token::BracketClose)(input)
}

// UnsignedLong(u64),
fn unsigned_long(input: &str) -> IResult<&str, Token> {
    // digit1：匹配至少一个数字字符（0-9）。
    // map：将匹配到的数字字符串转换为 i64 类型的数字，并封装成 Token::Number。
    map(digit1, |s: &str| Token::UnsignedLong(s.parse().unwrap()))(input)
}

// 最终解析器：匹配 '['，然后匹配一到多个解析器E，最后匹配 ']'
fn bracket_string(input: &str) -> IResult<&str, Token> {
    map(
        delimited(
            bracket_open,
            many1(alt((is_not("]"), tag("]]")))),
            bracket_close,
        ),
        |content: Vec<&str>| {
            // 合并解析出的内容，将其转化为 BracketString 类型的 Token
            // Token::BracketString(content.into_iter().collect::<Vec<String>>().join(""))

            // for nn_str in content {
            //     println!("------------------+++++++++++++++++++ {:?}", nn_str);
            // }

            let ooppoo = content.into_iter().collect::<Vec<&str>>().join("");

            println!("8080------------------+ ooppoo: {}", ooppoo);

            // Token::BracketString(String::from("XXXXXXXXXXXXXXXXXX ccc"))
            Token::BracketString(ooppoo)
        },
    )(input)
}

// 词法解析函数
pub fn lex(input: &str) -> IResult<&str, Vec<Token>> {
    many1(alt((
        keyword, // number,
        // identifier,
        bracket_string,
        whitespace,
        semicolon,
        comma,
        paren_open,
        paren_close,
        brace_open,
        brace_close,
        dot,
        ampersand,
        bracket_open,
        bracket_close,
        unsigned_long,
    )))(input)
}

// 测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdx() {
        let mdx = r###"
select 
{
    ( &600000000000005[>[[[[[[[[[[[[[[[[[[[[[[[[ ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]] ]]<].&300000000004670[].&1111111111[] ),
    (),
    (),
} ON ROws,
{} on columNS 
fRoM &0000099999[[one[[[ Cube ]] %^&];
;;;;;;;;;;;;;;;;;;;;;;;
"###;

        let mdx = r###"
select
{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004692[企业客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
}
on rows,
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
on columns
from &500000000000001;
        ;;;;;;
"###;

        let result = lex(mdx);

        if result.is_err() {
            // 如果解析失败，打印错误信息
            println!("Parsing the MDX failed with error: {:?}", result);
        }

        assert!(result.is_ok());

        let tokens = result.unwrap().1;

        // // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! printing for testing, can be deleted later
        // for kkk in &tokens {
        //     println!("KKKKKKKKKKKKKKKKKKKKKKKKKKK {:?}", kkk);
        // }
        // // ????????????????????????????????

        // 判断最后五个tokens是否符合四个分号和一个换行符
        let last_five_tokens = &tokens[tokens.len().saturating_sub(5)..];
        let expected_tokens = vec![
            Token::Semicolon,
            Token::Semicolon,
            Token::Semicolon,
            Token::Semicolon,
            Token::Whitespace,
        ];
        assert_eq!(last_five_tokens, &expected_tokens[..]);

        println!("\n\n---------------------------------------------Tokens of MDX:");
        // 循环打印每个 token
        for token in &tokens {
            println!("{:?}", token);
        }
        println!("---------------------------------------------\n\n");
        println!("Pass >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> 0 MDX");
    }
}
