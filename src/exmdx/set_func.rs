// use futures::future::BoxFuture;
// use crate::mdx_ast::ToCellValue;
// use crate::mdx_ast::{AstExpression, AstSegments, AstSet, Materializable};
// use crate::mdd::OlapVectorCoordinate;
// use crate::mdd::{CellValue, MultiDimensionalContext, MultiDimensionalEntity, Tuple};
// use crate::calcul;

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFnBottomPercent {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstSetFnCrossJoin {}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFnDescendants {
    Simple,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFnExcept {
    Simple,
}
