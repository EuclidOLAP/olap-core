use crate::mdd;

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

#[derive(Clone, Debug, PartialEq)]
pub enum AstAxis {
    Def{
        ast_set: AstSet,
        pos: u64
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstSelectionStatement {
    pub axes: Vec<AstAxis>,
    pub cube: Vec<AstSeg>,
    pub basic_slice: Option<AstTuple>,
}

impl AstSelectionStatement {
    pub fn build_axes(&self) -> Vec<mdd::Axis> {
        println!("build_axes .. . ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");

        let axes_count = self.axes.len();
        println!(">>> axes_count >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>: {}", axes_count);

        let mut axes:Vec<mdd::Axis> = Vec::with_capacity(axes_count);

        for i in 0..axes_count {
            let axis = mdd::Axis {
                pos_num: i as u32,
            };
            axes.push(axis);
        }

        axes
    }
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
