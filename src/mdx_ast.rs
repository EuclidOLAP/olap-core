#[derive(Clone, Debug, PartialEq)]
pub enum ExtMDXStatement {
    Querying {
        basic_cube: AstCube,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstCube {

}

#[derive(Clone, Debug, PartialEq)]
pub struct AstSeg {
    pub gid: Option<u64>,
    pub seg_str: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSegments {
    Segs(Vec<AstSeg>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstTuple {
    SegsList(Vec<AstSegments>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSet {
    Tuples(Vec<AstTuple>),
}



// #[derive(Clone, Debug, PartialEq)]
// pub enum Statement {
//     Variable {
//         name: String,
//         value: Box<Expression>,
//     },
//     Print {
//         value: Box<Expression>,
//     },
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum Expression {
//     Integer(i64),
//     Variable(String),
//     BinaryOperation {
//         lhs: Box<Expression>,
//         operator: Operator,
//         rhs: Box<Expression>,
//     },
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum Operator {
//     Add,
//     Sub,
//     Mul,
//     Div,
//     #[cfg(feature = "bit")]
//     Shl,
// }
