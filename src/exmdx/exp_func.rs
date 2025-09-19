use futures::future::BoxFuture;

use crate::exmdx::ast::ToVectorValue;
use crate::exmdx::ast::{AstSet, AstTuple};

use crate::exmdx::mdd::TupleVector;
use crate::mdd::{VectorValue, MultiDimensionalContext, MultiDimensionalEntity};

use crate::calcul;

use core::panic;

use crate::mdx_grammar::MdxStatementParser;
use crate::mdx_lexer::Lexer as MdxLexer;

use crate::exmdx::ast::{AstSegsObj, Materializable};

use crate::exmdx::logic::{AstBoolExp, ToBoolValue};

use crate::mdd::MemberRole;

use crate::meta_cache;

use crate::exmdx::ast::AstExpression;

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFunction {
    Avg(AstNumFnAvg),
    Count(AstNumFnCount),
    IIf(AstNumFnIIf),
    LookupCube(AstExpFnLookupCube),
    Name(AstStrFnName),
    Sum(AstNumFnSum),
    Max(AstNumFnMax),
    Min(AstNumFnMin),
    Abs(AstNumFnAbs),
    Aggregate(AstNumFnAggregate),
    CoalesceEmpty(AstNumFnCoalesceEmpty),
    Correlation(AstNumFnCorrelation),
    Covariance(AstNumFnCovariance),
    LinRegIntercept(AstNumFnLinRegIntercept),
    LinRegR2(AstNumFnLinRegR2),
    LinRegSlope(AstNumFnLinRegSlope),
    LinRegVariance(AstNumFnLinRegVariance),
    Median(AstNumFnMedian),
    Ordinal(AstNumFnOrdinal),
    Rank(AstNumFnRank),
    Stdev(AstNumFnStdev),
    Var(AstNumFnVar),
}

impl ToVectorValue for AstExpFunction {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move {
            match self {
                AstExpFunction::Avg(avg_fn) => avg_fn.val(slice_tuple, context, outer_param).await,
                AstExpFunction::Count(count_fn) => {
                    count_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::IIf(iif_fn) => iif_fn.val(slice_tuple, context, outer_param).await,
                AstExpFunction::Name(name_fn) => {
                    name_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::LookupCube(lookup_cube) => {
                    lookup_cube.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Sum(exp_fn_sum) => {
                    exp_fn_sum.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Max(exp_fn_max) => {
                    exp_fn_max.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Min(exp_fn_min) => {
                    exp_fn_min.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Abs(exp_fn) => exp_fn.val(slice_tuple, context, outer_param).await,
                AstExpFunction::Aggregate(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::CoalesceEmpty(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Correlation(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Covariance(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::LinRegIntercept(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::LinRegR2(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::LinRegSlope(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::LinRegVariance(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Median(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Ordinal(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Rank(exp_fn) => exp_fn.val(slice_tuple, context, outer_param).await,
                AstExpFunction::Stdev(exp_fn) => {
                    exp_fn.val(slice_tuple, context, outer_param).await
                }
                AstExpFunction::Var(exp_fn) => exp_fn.val(slice_tuple, context, outer_param).await,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstStrFnName {
    Chain,
    SegsObj(AstSegsObj),
    // OuterParam(Box<MultiDimensionalEntity>),
}

impl ToVectorValue for AstStrFnName {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move {
            let param_olap_obj = match self {
                AstStrFnName::SegsObj(segs) => segs.materialize(slice_tuple, context).await,
                AstStrFnName::Chain => {
                    if let Some(olap_obj) = outer_param {
                        olap_obj
                    } else {
                        panic!("[dsuc-0-fff2] AstStrFnName::val()")
                    }
                }
            };

            if let MultiDimensionalEntity::MemberRoleWrap(member_role) = param_olap_obj {
                match member_role {
                    MemberRole::BaseMember { member, .. } => VectorValue::Str(member.name.clone()),
                    _ => VectorValue::Str("name函数参数错误".to_string()),
                }
            } else {
                VectorValue::Str("name函数参数错误".to_string())
            }
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnAvg {
    Chain,
    AstSet(AstSet),
    AstSet_Exp(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnAvg {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Str("avg函数有待实现".to_string()) })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnCount {
    Chain,
    AstSet(AstSet),
    // OuterParam(Set),
}

impl ToVectorValue for AstNumFnCount {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move {
            if let Some(MultiDimensionalEntity::SetWrap(set)) = outer_param {
                VectorValue::Double(set.tuples.len() as f64)
            } else {
                VectorValue::Str("count函数参数错误".to_string())
            }
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnLookupCube {
    Chain(AstExpression),
    CubeSegs_Exp(AstSegsObj, AstExpression),
}

impl ToVectorValue for AstExpFnLookupCube {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move {
            let param_cube;
            let param_exp;

            match self {
                AstExpFnLookupCube::Chain(exp) => {
                    param_cube = if let Some(olap_obj) = outer_param {
                        match olap_obj {
                            MultiDimensionalEntity::Cube(cube) => cube,
                            _ => panic!("[dsuc-0-8492] AstExpFunction::val()"),
                        }
                    } else {
                        todo!("AstExpFunction::val() - lookupCube :: [SHUA-927381]")
                    };
                    param_exp = exp.clone();
                }
                AstExpFnLookupCube::CubeSegs_Exp(segs_obj, exp) => {
                    param_cube = if let MultiDimensionalEntity::Cube(cube) =
                        segs_obj.materialize(slice_tuple, context).await
                    {
                        cube
                    } else {
                        panic!("[dsuc-0-8492] AstExpFunction::val()")
                    };
                    param_exp = exp.clone();
                }
            }

            let mdx_with_str = meta_cache::mdx_formula_members_fragment(&param_cube);
            let tunnel_mdx = format!(
                "with\n{}\nSelect {{ ( &0 ) }} on rows\nfrom &{}",
                mdx_with_str, param_cube.gid
            );

            let tunnel_ast = MdxStatementParser::new()
                .parse(MdxLexer::new(&tunnel_mdx))
                .unwrap();
            let mut tunnel_context = tunnel_ast.gen_md_context(context.user_acol.clone()).await;
            // [warning] !!! look above code, the method - 'context.user_acol.clone()' may cause performance issue.
            // please consider to use '&context.user_acol'


            param_exp
                .val(
                    &tunnel_context.query_slice_tuple.clone(),
                    &mut tunnel_context,
                    None,
                )
                .await
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstNumFnIIf {
    pub bool_exp: AstBoolExp,
    pub true_exp: AstExpression,
    pub false_exp: AstExpression,
}

impl ToVectorValue for AstNumFnIIf {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move {
            let bool_val = self.bool_exp.bool_val(slice_tuple, context).await;
            if bool_val {
                self.true_exp.val(slice_tuple, context, None).await
            } else {
                self.false_exp.val(slice_tuple, context, None).await
            }
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnSum {
    Chain,
    AstSet(AstSet),
    AstSet_Exp(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnSum {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move {
            let mut outer_param = outer_param;
            let mut exp_opt: Option<AstExpression> = None;
            match self {
                AstNumFnSum::Chain => {
                    // do nothing
                }
                // AstNumFnSum::SegsSet(segs) => {
                //     outer_param = Some(segs.materialize(slice_tuple, context).await);
                // }
                // AstNumFnSum::SegsSetExp(segs, exp) => {
                //     outer_param = Some(segs.materialize(slice_tuple, context).await);
                //     exp_opt = Some(exp.clone());
                // }
                AstNumFnSum::AstSet(ast_set) => {
                    outer_param = Some(ast_set.materialize(slice_tuple, context).await);
                }
                AstNumFnSum::AstSet_Exp(ast_set, exp) => {
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
                    let ovc = TupleVector {
                        member_roles: tup.member_roles,
                    };
                    let values = calcul::calculate(vec![ovc], context).await;
                    let value = values[0].clone();
                    cell_vals.push(value);
                }
            }

            let sum: VectorValue = cell_vals
                .iter()
                .skip(1) // 跳过第 0 个元素
                .fold(cell_vals[0].clone(), |acc, val| acc + val.clone());

            sum
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnMax {
    Chain,
    AstSet(AstSet),
    AstSet_Exp(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnMax {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Str(String::from("// todo: Max()")) })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnMin {
    Chain,
    AstSet(AstSet),
    AstSet_Exp(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnMin {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Str(String::from("// todo: Min()")) })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnAbs {
    Chain,
    AstExp(AstExpression),
}

impl ToVectorValue for AstNumFnAbs {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnAggregate {
    Chain,
    AstSet_AstExp(AstSet, Option<AstExpression>),
}

impl ToVectorValue for AstNumFnAggregate {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct AstNumFnCoalesceEmpty {
    pub exps: Vec<AstExpression>,
}

impl ToVectorValue for AstNumFnCoalesceEmpty {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnCorrelation {
    Chain_NumExpY(AstExpression),
    AstSet_NumExpY_NumExpX(AstSet, AstExpression, AstExpression),
    AstSet_NumExpY(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnCorrelation {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnCovariance {
    Chain_NumExpY(AstExpression),
    AstSet_NumExpY_NumExpX(AstSet, AstExpression, AstExpression),
    AstSet_NumExpY(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnCovariance {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnLinRegIntercept {
    Chain_NumExpY(AstExpression),
    AstSet_NumExpY_NumExpX(AstSet, AstExpression, AstExpression),
    AstSet_NumExpY(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnLinRegIntercept {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnLinRegR2 {
    Chain_NumExpY(AstExpression),
    AstSet_NumExpY_NumExpX(AstSet, AstExpression, AstExpression),
    AstSet_NumExpY(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnLinRegR2 {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnLinRegSlope {
    Chain_NumExpY(AstExpression),
    AstSet_NumExpY_NumExpX(AstSet, AstExpression, AstExpression),
    AstSet_NumExpY(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnLinRegSlope {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnLinRegVariance {
    Chain_NumExpY(AstExpression),
    AstSet_NumExpY_NumExpX(AstSet, AstExpression, AstExpression),
    AstSet_NumExpY(AstSet, AstExpression),
}

impl ToVectorValue for AstNumFnLinRegVariance {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnMedian {
    Chain,
    AstSet_AstExp(AstSet, Option<AstExpression>),
}

impl ToVectorValue for AstNumFnMedian {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnOrdinal {
    Chain,
    LevelSegs(AstSegsObj),
}

impl ToVectorValue for AstNumFnOrdinal {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnRank {
    Chain_AstSet(AstSet),
    AstTuple_AstSet_AstExp(AstTuple, AstSet, AstExpression),
    AstTuple_AstSet(AstTuple, AstSet),
}

impl ToVectorValue for AstNumFnRank {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnStdev {
    Chain,
    AstSet_AstExp(AstSet, Option<AstExpression>),
}

impl ToVectorValue for AstNumFnStdev {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnVar {
    Chain,
    AstSet_AstExp(AstSet, Option<AstExpression>),
}

impl ToVectorValue for AstNumFnVar {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, VectorValue> {
        Box::pin(async move { VectorValue::Null })
    }
}
