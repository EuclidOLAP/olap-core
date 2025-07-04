use futures::future::BoxFuture;

use crate::mdx_ast::{AstExpression, AstSegments, AstSet};
use crate::mdx_ast::ToCellValue;

use crate::mdd::{MultiDimensionalContext, Tuple, CellValue, MultiDimensionalEntity};

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFuncSum {
    Simple,
    SegsSet(AstSegments),
    SegsSetExp(AstSegments, AstExpression),
    BraceSet(AstSet),
    BraceSetExp(AstSet, AstExpression),
}

impl ToCellValue for AstExpFuncSum {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a Tuple,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            CellValue::Str(String::from("// todo: Sum()"))
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFuncMax {
    Simple,
    SegsSet(AstSegments),
    SegsSetExp(AstSegments, AstExpression),
    BraceSet(AstSet),
    BraceSetExp(AstSet, AstExpression),
}

impl ToCellValue for AstExpFuncMax {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a Tuple,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            CellValue::Str(String::from("// todo: Max()"))
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFuncMin {
    Simple,
    SegsSet(AstSegments),
    SegsSetExp(AstSegments, AstExpression),
    BraceSet(AstSet),
    BraceSetExp(AstSet, AstExpression),
}

impl ToCellValue for AstExpFuncMin {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a Tuple,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            CellValue::Str(String::from("// todo: Min()"))
        })
    }
}
