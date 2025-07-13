use futures::future::BoxFuture;

use crate::mdx_ast::ToCellValue;
use crate::mdx_ast::{AstExpression, AstSet, Materializable};

use crate::exmdx::ast::AstSegsObj;

use crate::exmdx::mdd::TupleVector;
use crate::mdd::{CellValue, MultiDimensionalContext, MultiDimensionalEntity};

use crate::calcul;

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFuncSum {
    Simple,
    SegsSet(AstSegsObj),
    SegsSetExp(AstSegsObj, AstExpression),
    BraceSet(AstSet),
    BraceSetExp(AstSet, AstExpression),
}

impl ToCellValue for AstExpFuncSum {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            let mut outer_param = outer_param;
            let mut exp_opt: Option<AstExpression> = None;
            match self {
                AstExpFuncSum::Simple => {
                    // do nothing
                }
                AstExpFuncSum::SegsSet(segs) => {
                    outer_param = Some(segs.materialize(slice_tuple, context).await);
                }
                AstExpFuncSum::SegsSetExp(segs, exp) => {
                    outer_param = Some(segs.materialize(slice_tuple, context).await);
                    exp_opt = Some(exp.clone());
                }
                AstExpFuncSum::BraceSet(ast_set) => {
                    outer_param = Some(ast_set.materialize(slice_tuple, context).await);
                }
                AstExpFuncSum::BraceSetExp(ast_set, exp) => {
                    outer_param = Some(ast_set.materialize(slice_tuple, context).await);
                    exp_opt = Some(exp.clone());
                }
            }

            let set_param = outer_param.unwrap();
            let set;
            if let MultiDimensionalEntity::SetWrap(s) = set_param {
                set = s;
            } else {
                panic!("Invalid parameter for Sum() function")
            }
            let mut cell_vals = Vec::new();
            for tuple in set.tuples {
                let tup = slice_tuple.merge(&tuple);
                if let Some(ref exp) = exp_opt {
                    let value = exp.val(&tup, context, None).await;
                    cell_vals.push(value);
                } else {
                    let ovc = TupleVector { member_roles: tup.member_roles };
                    let values = calcul::calculate(vec![ovc], context).await;
                    let value = values[0].clone();
                    cell_vals.push(value);
                }
            }

            let sum: CellValue = cell_vals
                .iter()
                .skip(1) // 跳过第 0 个元素
                .fold(cell_vals[0].clone(), |acc, val| acc + val.clone());

            sum
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFuncMax {
    Simple,
    SegsSet(AstSegsObj),
    SegsSetExp(AstSegsObj, AstExpression),
    BraceSet(AstSet),
    BraceSetExp(AstSet, AstExpression),
}

impl ToCellValue for AstExpFuncMax {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: Max()")) })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFuncMin {
    Simple,
    SegsSet(AstSegsObj),
    SegsSetExp(AstSegsObj, AstExpression),
    BraceSet(AstSet),
    BraceSetExp(AstSet, AstExpression),
}

impl ToCellValue for AstExpFuncMin {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: Min()")) })
    }
}
