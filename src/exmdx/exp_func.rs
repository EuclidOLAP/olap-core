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

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnAbs {
    Simple,
}
impl ToCellValue for AstExpFnAbs {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Abs()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnAggregate {
    Simple,
}
impl ToCellValue for AstExpFnAggregate {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Aggregate()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCalculationCurrentPass {
    Simple,
}
impl ToCellValue for AstExpFnCalculationCurrentPass {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            CellValue::Str(String::from("// todo: ExpFn - CalculationCurrentPass()"))
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCalculationPassValue {
    Simple,
}
impl ToCellValue for AstExpFnCalculationPassValue {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(
            async move { CellValue::Str(String::from("// todo: ExpFn - CalculationPassValue()")) },
        )
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCellValue {
    Simple,
}
impl ToCellValue for AstExpFnCellValue {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - CellValue()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCoalesceEmpty {
    Simple,
}
impl ToCellValue for AstExpFnCoalesceEmpty {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - CoalesceEmpty()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCorrelation {
    Simple,
}
impl ToCellValue for AstExpFnCorrelation {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Correlation()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCovariance {
    Simple,
}
impl ToCellValue for AstExpFnCovariance {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Covariance()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCovarianceN {
    Simple,
}
impl ToCellValue for AstExpFnCovarianceN {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - CovarianceN()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnDateDiff {
    Simple,
}
impl ToCellValue for AstExpFnDateDiff {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - DateDiff()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnDatePart {
    Simple,
}
impl ToCellValue for AstExpFnDatePart {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - DatePart()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnDistinctCount {
    Simple,
}
impl ToCellValue for AstExpFnDistinctCount {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - DistinctCount()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnEnumText {
    Simple,
}
impl ToCellValue for AstExpFnEnumText {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - EnumText()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnEnumValue {
    Simple,
}
impl ToCellValue for AstExpFnEnumValue {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - EnumValue()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnExp {
    Simple,
}
impl ToCellValue for AstExpFnExp {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Exp()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnFactorial {
    Simple,
}
impl ToCellValue for AstExpFnFactorial {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Factorial()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnInStr {
    Simple,
}
impl ToCellValue for AstExpFnInStr {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - InStr()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnInt {
    Simple,
}
impl ToCellValue for AstExpFnInt {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Int()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLen {
    Simple,
}
impl ToCellValue for AstExpFnLen {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Len()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLinRegIntercept {
    Simple,
}
impl ToCellValue for AstExpFnLinRegIntercept {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - LinRegIntercept()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLinRegPoint {
    Simple,
}
impl ToCellValue for AstExpFnLinRegPoint {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - LinRegPoint()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLinRegR2 {
    Simple,
}
impl ToCellValue for AstExpFnLinRegR2 {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - LinRegR2()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLinRegSlope {
    Simple,
}
impl ToCellValue for AstExpFnLinRegSlope {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - LinRegSlope()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLinRegVariance {
    Simple,
}
impl ToCellValue for AstExpFnLinRegVariance {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - LinRegVariance()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLn {
    Simple,
}
impl ToCellValue for AstExpFnLn {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Ln()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLog {
    Simple,
}
impl ToCellValue for AstExpFnLog {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Log()")) })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLog10 {
    Simple,
}
impl ToCellValue for AstExpFnLog10 {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: ExpFn - Log10()")) })
    }
}
