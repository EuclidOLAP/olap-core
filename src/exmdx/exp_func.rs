use futures::future::BoxFuture;

use crate::exmdx::ast::AstSet;
use crate::exmdx::ast::ToCellValue;

use crate::exmdx::mdd::TupleVector;
use crate::mdd::{CellValue, MultiDimensionalContext, MultiDimensionalEntity};

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
}

impl ToCellValue for AstExpFunction {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
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

impl ToCellValue for AstStrFnName {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
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
                    MemberRole::BaseMember { member, .. } => CellValue::Str(member.name.clone()),
                    _ => CellValue::Str("name函数参数错误".to_string()),
                }
            } else {
                CellValue::Str("name函数参数错误".to_string())
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

impl ToCellValue for AstNumFnAvg {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str("avg函数有待实现".to_string()) })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnCount {
    Chain,
    AstSet(AstSet),
    // OuterParam(Set),
}

impl ToCellValue for AstNumFnCount {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            if let Some(MultiDimensionalEntity::SetWrap(set)) = outer_param {
                CellValue::Double(set.tuples.len() as f64)
            } else {
                CellValue::Str("count函数参数错误".to_string())
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

impl ToCellValue for AstExpFnLookupCube {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
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
            let mut tunnel_context = tunnel_ast.gen_md_context().await;

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

impl ToCellValue for AstNumFnIIf {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
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

impl ToCellValue for AstNumFnSum {
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

            let sum: CellValue = cell_vals
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

impl ToCellValue for AstNumFnMax {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: Max()")) })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstNumFnMin {
    Chain,
    AstSet(AstSet),
    AstSet_Exp(AstSet, AstExpression),
}

impl ToCellValue for AstNumFnMin {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str(String::from("// todo: Min()")) })
    }
}
