// mdx_parser/lexer.rs

use nom::{
    // alt：这是一个组合子（combinator），它用于尝试多个解析规则。如果第一个规则失败，它会继续尝试下一个规则，直到找到匹配。
    branch::alt,

    // is_not：这个解析器会匹配输入中一段不包含指定字符的内容。例如，is_not(" ,(){}[]") 会匹配所有不是逗号、空格、括号等的字符。
    // tag：匹配一个固定的字符串或字符。在这里它用于匹配特定的符号或关键字。
    bytes::complete::{is_not, tag},

    // alpha1：匹配至少一个字母字符。
    // digit1：匹配至少一个数字字符。
    // space1：匹配至少一个空格字符。
    character::complete::{alpha1, digit1, space1},

    // map：将解析的结果转换为另一种形式。在这里，它将匹配的字符串转换为 Token 枚举。
    combinator::map,

    // 关于error::ParseError参考下面的url
    // https://www.cnblogs.com/Tifahfyf/p/18617780
    error::ParseError,

    // many1：用于匹配一次或多次某个解析器，返回一个 Vec（例如，多个 token）。
    multi::many1,

    // tuple：可以用来匹配一系列解析器并返回它们的结果。
    sequence::tuple,

    // IResult：是 nom 的一个类型，表示解析的结果。它通常包含解析剩余输入和解析成功的值。
    IResult,
};

// 定义 Token 类型
#[derive(Debug, PartialEq, Clone)]
// Token 枚举类型定义了我们解析出的各种 token。每个变体代表一个词法单元（如关键字、标识符、数字、符号等）。
pub enum Token {
    // Keyword(String)：存储关键字，如 "SELECT" 或 "FROM"。
    Keyword(String),
    // Identifier(String)：存储标识符（例如变量名、函数名等）。
    Identifier(String),
    // Number(i64)：存储数字。
    Number(i64),
    Comma,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    Dot,
    // 你可以继续添加更多 Token 类型
}

// 词法解析函数：解析 MDX 查询中的关键字
// 每个词法解析函数用于识别输入中的特定部分，并将其转换为一个 Token。
// 解析关键字（keyword）
fn keyword(input: &str) -> IResult<&str, Token> {
    // alt：首先尝试匹配 "SELECT"、"FROM"、"WHERE" 等关键字。
    alt((
        // tag：用于匹配精确的字符串（如 "SELECT"）。
        // map：一旦匹配成功，将其转换为相应的 Token::Keyword 枚举，并返回。
        map(tag("SELECT"), |_| Token::Keyword("SELECT".to_string())),
        map(tag("FROM"), |_| Token::Keyword("FROM".to_string())),
        map(tag("WHERE"), |_| Token::Keyword("WHERE".to_string())),
        // 你可以继续添加更多的关键字

        // input：输入字符串。
    ))(input)
}

// 词法解析函数：解析标识符（如 [Measures].[sales count]）
// 解析标识符（identifier）
fn identifier(input: &str) -> IResult<&str, Token> {
    // is_not(" ,(){}[]")：匹配不是空格、逗号、括号、花括号、方括号等的任意字符。这意味着标识符可以是由字母、数字和下划线组成的字符串，但不能包含这些符号。
    // map：将匹配的结果（一个标识符字符串）转换为 Token::Identifier。
    map(is_not(" ,(){}[]"), |s: &str| {
        Token::Identifier(s.to_string())
    })(input)
}

// 词法解析函数：解析数字
// 解析数字（number）
fn number(input: &str) -> IResult<&str, Token> {
    // digit1：匹配至少一个数字字符（0-9）。
    // map：将匹配到的数字字符串转换为 i64 类型的数字，并封装成 Token::Number。
    map(digit1, |s: &str| Token::Number(s.parse().unwrap()))(input)
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

// 组合所有的词法解析函数
// lex 函数的输出是一个 IResult，其中包含成功的 token 列表（例如 Token::Keyword("SELECT")）或剩余的未解析部分（如果有的话）。
pub fn lex(input: &str) -> IResult<&str, Vec<Token>> {
    // many1：匹配一次或多次提供的解析器组合。这里的组合包括关键字、标识符、数字、符号等。
    // alt：尝试逐个规则，直到找到匹配的规则。
    many1(alt((
        keyword,
        identifier,
        number,
        comma,
        paren_open,
        paren_close,
        brace_open,
        brace_close,
        dot,
        // 可以继续添加更多的词法规则
    )))(input)
}

// #[cfg(test)]：这个模块仅在运行测试时才会被编译和执行。
#[cfg(test)]
mod tests {
    use super::*;

    // test_lex_keyword：测试解析关键字 "SELECT"，确保 lex 函数正确地将其转换为 Token::Keyword("SELECT")。
    #[test]
    fn test_lex_keyword() {
        let input = "SELECT";
        let result = lex(input);
        assert_eq!(result, Ok(("", vec![Token::Keyword("SELECT".to_string())])));
    }

    #[test]
    fn test_lex_identifier() {
        let input = "[Measures].[sales count]";
        let result = lex(input);
        // assert_eq!(result, Ok(("", vec![Token::Identifier("[Measures].[sales count]".to_string())])));
    }

    // #[test]
    // fn test_lex_number() {
    //     let input = "12345";
    //     let result = lex(input);
    //     assert_eq!(result, Ok(("", vec![Token::Number(12345)])));
    // }
}
