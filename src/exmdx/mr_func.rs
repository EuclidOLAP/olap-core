// use futures::future::BoxFuture;
// use crate::mdx_ast::ToCellValue;
// use crate::mdx_ast::{AstExpression, AstSegments, AstSet, Materializable};
// use crate::mdd::OlapVectorCoordinate;
// use crate::mdd::{CellValue, MultiDimensionalContext, MultiDimensionalEntity, Tuple};
// use crate::calcul;

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnFirstChild {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnFirstSibling {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnLag {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnLastChild {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnLastSibling {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnLead {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnParallelPeriod {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnPrevMember {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnNextMember {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnAncestor {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnCousin {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnDefaultMember {
    Simple,
}
