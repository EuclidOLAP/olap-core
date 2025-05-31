use logos::Logos;
use std::fmt; // to implement the Display trait
use std::num::{ParseFloatError, ParseIntError};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    InvalidDouble(ParseFloatError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

impl From<ParseFloatError> for LexicalError {
    fn from(err: ParseFloatError) -> Self {
        LexicalError::InvalidDouble(err)
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+", skip r"#.*\n?", skip r"--.*\n?", error = LexicalError)]
pub enum Token {
    #[regex("(?i)With")]
    With,
    #[regex("(?i)Member")]
    Member,
    #[regex("(?i)Set")]
    Set,
    #[regex("(?i)As")]
    As,

    #[regex("(?i)select")]
    Select,
    #[regex("(?i)on")]
    On,
    #[regex("(?i)from")]
    From,
    #[regex("(?i)where")]
    Where,

    #[regex(r"\[(?:[^\]]|\]\])*\]", |lex| {
        let raw = lex.slice();
        raw[1..raw.len() - 1].replace("]]", "]")
    })]
    BracketedString(String),
    #[regex(r#""[^"]*""#, |lex| {
        let raw = lex.slice();
        raw[1..raw.len() - 1].to_string()
    })]
    QuotedString(String),

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiplied,
    #[token("/")]
    Divided,

    #[token("{")]
    CurlyBraceLeft,
    #[token("}")]
    CurlyBraceRight,

    #[token("(")]
    RoundBracketLeft,
    #[token(")")]
    RoundBracketRight,

    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("&")]
    Ampersand,
    #[token(";")]
    Semicolon,

    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Integer(u64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse())]
    Double(f64),

    #[regex("(?i)COLUMNS")]
    Columns,
    #[regex("(?i)ROWS")]
    Rows,
    #[regex("(?i)PAGES")]
    Pages,
    #[regex("(?i)CHAPTERS")]
    Chapters,
    #[regex("(?i)SECTIONS")]
    Sections,

    #[regex("(?i)Parent")]
    Parent,

    #[regex("(?i)Children")]
    Children,

    #[regex("(?i)Avg")]
    Avg,
    #[regex("(?i)Count")]
    Count,

    #[regex("(?i)ClosingPeriod")]
    ClosingPeriod,

    #[regex("(?i)OpeningPeriod")]
    OpeningPeriod,

    #[regex("(?i)Level")]
    Level,

    #[regex("(?i)Levels")]
    Levels,

    #[regex("(?i)IIf")]
    IIf,

    #[regex("(?i)Not")]
    Not,

    #[regex("(?i)Or")]
    Or,
    #[regex("(?i)And")]
    And,

    #[token("<")]   // Less Than
    LT,
    #[token("<=")]  // Less Than or Equal To
    LE,
    #[token("=")]   // Equal To
    EQ,
    #[token("<>")]  // Not Equal To
    NE,
    #[token(">")]   // Greater Than
    GT,
    #[token(">=")]  // Greater Than or Equal To
    GE,

    #[regex("(?i)IsLeaf")]
    IsLeaf,

    #[regex("(?i)Name")]
    Name,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
