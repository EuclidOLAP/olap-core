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
    // #####################################################
    // ##  Primary Keywords                               ##
    // #####################################################
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

    #[regex("(?i)Case")]
    Case,
    #[regex("(?i)When")]
    When,
    #[regex("(?i)Then")]
    Then,
    #[regex("(?i)Else")]
    Else,
    #[regex("(?i)End")]
    End,

    // #####################################################
    // ##  String, Identifier and Numeric                 ##
    // #####################################################
    #[regex(r"[a-zA-Z_][a-zA-Z_0-9]*", |lex| {
        String::from(lex.slice())
    })]
    Identifier(String),
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
    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Integer(u64),
    #[regex(r"-[0-9]+", |lex| lex.slice().parse())]
    NegativeInteger(i64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse())]
    Double(f64),

    // #####################################################
    // ##  Special Characters                             ##
    // #####################################################
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiplied,
    #[token("/")]
    Divided,
    #[token("%")]
    Percent,

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

    // #####################################################
    // ##  Member(Role) functions                         ##
    // #####################################################
    #[regex("(?i)Parent")]
    Parent,
    #[regex("(?i)CurrentMember")]
    CurrentMember,
    #[regex("(?i)ClosingPeriod")]
    ClosingPeriod,
    #[regex("(?i)FirstChild")]
    FirstChild,
    #[regex("(?i)FirstSibling")]
    FirstSibling,
    #[regex("(?i)Lag")]
    Lag,
    #[regex("(?i)LastChild")]
    LastChild,
    #[regex("(?i)LastSibling")]
    LastSibling,
    #[regex("(?i)Lead")]
    Lead,
    #[regex("(?i)OpeningPeriod")]
    OpeningPeriod,
    #[regex("(?i)ParallelPeriod")]
    ParallelPeriod,
    #[regex("(?i)PrevMember")]
    PrevMember,
    #[regex("(?i)NextMember")]
    NextMember,
    #[regex("(?i)Ancestor")]
    Ancestor,
    #[regex("(?i)Cousin")]
    Cousin,
    #[regex("(?i)DefaultMember")]
    DefaultMember,

    // #####################################################
    // ##  Set functions                                  ##
    // #####################################################
    #[regex("(?i)Children")]
    Children,
    #[regex("(?i)BottomPercent")]
    BottomPercent,
    #[regex("(?i)CrossJoin")]
    CrossJoin,
    #[regex("(?i)Descendants")]
    Descendants,
    #[regex("(?i)Except")]
    Except,
    #[regex("(?i)Filter")]
    Filter,
    #[regex("(?i)Intersect")]
    Intersect,
    #[regex("(?i)Members")]
    Members,
    #[regex("(?i)Order")]
    Order,
    #[regex("(?i)Tail")]
    Tail,
    #[regex("(?i)TopCount")]
    TopCount,
    #[regex("(?i)TopPercent")]
    TopPercent,
    #[regex("(?i)Union")]
    Union,
    #[regex("(?i)Ytd")]
    Ytd,
    #[regex("(?i)Qtd")]
    Qtd,
    #[regex("(?i)Distinct")]
    Distinct,
    #[regex("(?i)DrilldownLevel")]
    DrilldownLevel,
    #[regex("(?i)DrilldownLevelBottom")]
    DrilldownLevelBottom,
    #[regex("(?i)DrillDownLevelTop")]
    DrillDownLevelTop,
    #[regex("(?i)DrillDownMember")]
    DrillDownMember,
    #[regex("(?i)DrillDownMemberBottom")]
    DrillDownMemberBottom,
    #[regex("(?i)DrillDownMemberTop")]
    DrillDownMemberTop,
    #[regex("(?i)DrillupLevel")]
    DrillupLevel,
    #[regex("(?i)DrillupMember")]
    DrillupMember,
    #[regex("(?i)Ancestors")]
    Ancestors,
    #[regex("(?i)BottomCount")]
    BottomCount,
    #[regex("(?i)BottomSum")]
    BottomSum,
    #[regex("(?i)TopSum")]
    TopSum,
    #[regex("(?i)Extract")]
    Extract,
    #[regex("(?i)PeriodsToDate")]
    PeriodsToDate,
    #[regex("(?i)Generate")]
    Generate,
    #[regex("(?i)Head")]
    Head,
    #[regex("(?i)Subset")]
    Subset,

    // #####################################################
    // ##  Expression numeric functions                   ##
    // #####################################################
    #[regex("(?i)Avg")]
    Avg,
    #[regex("(?i)Sum")]
    Sum,
    #[regex("(?i)Max")]
    Max,
    #[regex("(?i)Min")]
    Min,
    #[regex("(?i)Count")]
    Count,
    #[regex("(?i)IIf")]
    IIf,
    #[regex("(?i)CoalesceEmpty")]
    CoalesceEmpty,
    #[regex("(?i)Aggregate")]
    Aggregate,
    #[regex("(?i)Median")]
    Median,
    #[regex("(?i)Rank")]
    Rank,
    #[regex("(?i)Abs")]
    Abs,
    #[regex("(?i)Correlation")]
    Correlation,
    #[regex("(?i)Covariance")]
    Covariance,
    #[regex("(?i)LinRegIntercept")]
    LinRegIntercept,
    #[regex("(?i)LinRegR2")]
    LinRegR2,
    #[regex("(?i)LinRegSlope")]
    LinRegSlope,
    #[regex("(?i)LinRegVariance")]
    LinRegVariance,
    #[regex("(?i)Stdev")]
    Stdev,
    #[regex("(?i)Var")]
    Var,
    #[regex("(?i)Ordinal")]
    Ordinal,

    // #####################################################
    // ##  Tunnel Functions                               ##
    // #####################################################
    #[regex("(?i)LookupCube")]
    LookupCube,

    // #####################################################
    // ##  Expression string functions                    ##
    // #####################################################
    #[regex("(?i)Name")]
    Name,

    // #####################################################
    // ##  Hierarchy(Role) Functions                      ##
    // #####################################################
    #[regex("(?i)Dimension")]
    Dimension,
    #[regex("(?i)Dimensions")]
    Dimensions,
    #[regex("(?i)Hierarchy")]
    Hierarchy,

    // #####################################################
    // ##  Level(Role) Functions                          ##
    // #####################################################
    #[regex("(?i)Level")]
    Level,
    #[regex("(?i)Levels")]
    Levels,
    #[regex("(?i)Generation")]
    Generation,
    #[regex("(?i)Generations")]
    Generations,

    // #####################################################
    // ##  Boolean Expression Keywords                    ##
    // #####################################################
    #[regex("(?i)Or")]
    Or,
    #[regex("(?i)And")]
    And,
    #[regex("(?i)Not")]
    Not,
    #[token("<")] // Less Than
    LT,
    #[token("<=")] // Less Than or Equal To
    LE,
    #[token("=")] // Equal To
    EQ,
    #[token("<>")] // Not Equal To
    NE,
    #[token(">")] // Greater Than
    GT,
    #[token(">=")] // Greater Than or Equal To
    GE,

    // #####################################################
    // ##  Boolean Expression Functions                   ##
    // #####################################################
    #[regex("(?i)IsLeaf")]
    IsLeaf,
    #[regex("(?i)IsEmpty")]
    IsEmpty,
    #[regex("(?i)IsAncestor")]
    IsAncestor,
    #[regex("(?i)IsGeneration")]
    IsGeneration,
    #[regex("(?i)IsSibling")]
    IsSibling,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
