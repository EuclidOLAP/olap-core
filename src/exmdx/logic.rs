use core::panic;

use futures::future::BoxFuture;

use crate::exmdx::ast::AstSegsObj;

use crate::exmdx::mdd::TupleVector;
use crate::mdd::MultiDimensionalContext;
use crate::mdd::{MemberRole, MultiDimensionalEntity};

use crate::mdx_ast::{AstExpression, Materializable, ToBoolValue, ToCellValue};

#[derive(Clone, Debug, PartialEq)]
pub struct AstBoolExp {
    pub terms: Vec<AstBoolTerm>,
}

impl ToBoolValue for AstBoolExp {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            for bool_term in &self.terms {
                let result = bool_term.bool_val(slice_tuple, context).await;
                if result {
                    return true;
                }
            }
            false
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstBoolTerm {
    pub factories: Vec<AstBoolFactory>,
    // BoolFactory(AstBoolFactory),
    // BoolTermAndBoolFactory(Box<AstBoolTerm>, AstBoolFactory),
}

impl ToBoolValue for AstBoolTerm {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            for factory in &self.factories {
                let result = factory.bool_val(slice_tuple, context).await;
                if !result {
                    return false; // 只要有一个为false，直接返回false
                }
            }
            true
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolFactory {
    BoolPrimary(AstBoolPrimary),
    Not_BoolPrimary(AstBoolPrimary),
    // ExpComparesExp(AstExpression, String, AstExpression),
    // BoolExp(Box<AstBoolExp>),
    // BoolFn(AstBoolFunction),
}

impl ToBoolValue for AstBoolFactory {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolFactory::BoolPrimary(bool_pri) => {
                    bool_pri.bool_val(slice_tuple, context).await
                }
                AstBoolFactory::Not_BoolPrimary(bool_pri) => {
                    !bool_pri.bool_val(slice_tuple, context).await
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolPrimary {
    ExpComparesExp(AstExpression, String, AstExpression),
    BoolExp(AstBoolExp),
    // BoolExp(Box<AstBoolExp>),
    BoolFn(AstBoolFunction),
}

impl ToBoolValue for AstBoolPrimary {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                Self::ExpComparesExp(exp1, op, exp2) => {
                    let val1 = exp1.val(slice_tuple, context, None).await;
                    let val2 = exp2.val(slice_tuple, context, None).await;
                    val1.logical_cmp(op, &val2)
                }
                Self::BoolExp(bool_exp) => bool_exp.bool_val(slice_tuple, context).await,
                Self::BoolFn(bool_fn) => bool_fn.bool_val(slice_tuple, context).await,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolFunction {
    IsLeaf(AstBoolFnIsLeaf),
}

impl ToBoolValue for AstBoolFunction {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolFunction::IsLeaf(is_leaf_fn) => {
                    is_leaf_fn.bool_val(slice_tuple, context).await
                }
            }
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolFnIsLeaf {
    Member_Segs(AstSegsObj),
}

// impl AstBoolFnIsLeaf {
//     pub fn new(member_segs: AstSegsObj) -> Self {
//         Self { member_segs }
//     }
// }

impl ToBoolValue for AstBoolFnIsLeaf {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            let mem_role_segs = match self {
                AstBoolFnIsLeaf::Member_Segs(segs) => segs,
            };

            let olap_obj = mem_role_segs.materialize(slice_tuple, context).await;

            if let MultiDimensionalEntity::MemberRoleWrap(member_role) = olap_obj {
                match member_role {
                    MemberRole::BaseMember { member, .. } => member.leaf,
                    _ => true,
                }
            } else {
                panic!("[hsju6679] The entity is not a MemberRole variant.");
            }

            // let olap_obj = self.member_segs.materialize(slice_tuple, context).await;
            // if let MultiDimensionalEntity::MemberRoleWrap(member_role) = olap_obj {
            //     match member_role {
            //         MemberRole::BaseMember { member, .. } => member.leaf,
            //         _ => true,
            //     }
            // } else {
            //     panic!("[hsju6679] The entity is not a MemberRole variant.");
            // }
        })
    }
}
