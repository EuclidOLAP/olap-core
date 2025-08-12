use crate::exmdx::ast::AstExpression;
use crate::exmdx::ast::AstSegsObj;

#[derive(Clone, Debug, PartialEq)]
pub enum AstHierarchyFunction {
    Dimension(AstHierFnDimension),
    Dimensions(AstHierFnDimensions),
    Hierarchy(AstHierFnHierarchy),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstHierFnDimension {
    Chain,
    OlapObj(AstSegsObj),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstHierFnDimensions {
    pub exp: AstExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstHierFnHierarchy {
    Chain,
    OlapObj(AstSegsObj),
}
