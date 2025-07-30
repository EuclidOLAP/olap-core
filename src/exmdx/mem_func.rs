use futures::future::BoxFuture;

use core::panic;

use crate::exmdx::ast::AstExpression;
use crate::exmdx::ast::AstSegsObj;

use crate::exmdx::ast::Materializable;
use crate::exmdx::mdd::TupleVector;
use crate::mdd::MultiDimensionalContext;
use crate::mdd::{MemberRole, MultiDimensionalEntity};
use crate::meta_cache;

pub trait MemberRoleAccess {
    fn resolve_member_role<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFunction {
    Parent(AstMemberFnParent),
    ClosingPeriod(AstMemberFnClosingPeriod),
    OpeningPeriod(AstMemberFnOpeningPeriod),
    CurrentMember(AstMemberFnCurrentMember),
    FirstChild(AstMemberFnFirstChild),
    FirstSibling(AstMemberFnFirstSibling),
    Lag(AstMemberFnLag),
    LastChild(AstMemberFnLastChild),
    LastSibling(AstMemberFnLastSibling),
    Lead(AstMemberFnLead),
    ParallelPeriod(AstMemberFnParallelPeriod),
    PrevMember(AstMemberFnPrevMember),
    NextMember(AstMemberFnNextMember),
    Ancestor(AstMemberFnAncestor),
    Cousin(AstMemberFnCousin),
    DefaultMember(AstMemberFnDefaultMember),
}

impl AstMemberFunction {
    pub async fn get_member(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match self {
            // CurrentMember()
            AstMemberFunction::CurrentMember(current_member) => {
                current_member
                    .do_get_member(left_outer_param, slice_tuple, context)
                    .await
            }
            // parent()
            AstMemberFunction::Parent(AstMemberFnParent::Chain) => {
                AstMemberFnParent::do_get_member(left_outer_param, context).await
            }
            AstMemberFunction::Parent(AstMemberFnParent::MemSegs(_segs)) => {
                todo!("AstMemberFunction::get_member()")
            }
            // ClosingPeriod()
            AstMemberFunction::ClosingPeriod(AstMemberFnClosingPeriod::Chain) => {
                AstMemberFnClosingPeriod::do_get_member(
                    left_outer_param,
                    None,
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::ClosingPeriod(AstMemberFnClosingPeriod::LvSegs(level_segs)) => {
                AstMemberFnClosingPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::ClosingPeriod(AstMemberFnClosingPeriod::LvSegs_MemSegs(
                level_segs,
                member_segs,
            )) => {
                AstMemberFnClosingPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    Some(member_segs),
                    slice_tuple,
                    context,
                )
                .await
            }
            // OpeningPeriod()
            AstMemberFunction::OpeningPeriod(AstMemberFnOpeningPeriod::Chain) => {
                AstMemberFnOpeningPeriod::do_get_member(
                    left_outer_param,
                    None,
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::OpeningPeriod(AstMemberFnOpeningPeriod::LvSegs(level_segs)) => {
                AstMemberFnOpeningPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::OpeningPeriod(AstMemberFnOpeningPeriod::LvSegs_MemSegs(
                level_segs,
                member_segs,
            )) => {
                AstMemberFnOpeningPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    Some(member_segs),
                    slice_tuple,
                    context,
                )
                .await
            } // _ => todo!("AstMemberFunction::get_member() - [NNNNNN-887766]"),
            Self::FirstChild(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::FirstSibling(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::Lag(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::LastChild(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::LastSibling(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::Lead(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::ParallelPeriod(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::PrevMember(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::NextMember(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::Ancestor(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::Cousin(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
            Self::DefaultMember(member_role_fn) => MultiDimensionalEntity::MemberRoleWrap(
                member_role_fn
                    .resolve_member_role(slice_tuple, context, left_outer_param)
                    .await,
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AstMemberFnClosingPeriod {
    Chain,
    LvSegs(AstSegsObj),
    LvSegs_MemSegs(AstSegsObj, AstSegsObj),
}

impl AstMemberFnClosingPeriod {
    async fn do_get_member(
        left_outer_param: Option<MultiDimensionalEntity>,
        level_param: Option<&AstSegsObj>,
        member_param: Option<&AstSegsObj>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match (left_outer_param, level_param, member_param) {
            (None, Some(lv_segs), None) => {
                let olap_obj = lv_segs.materialize(slice_tuple, context).await;
                if let MultiDimensionalEntity::LevelRole(lv_role) = olap_obj {
                    MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role: lv_role.dim_role.clone(),
                        member: meta_cache::get_member_by_gid(lv_role.level.closing_period_gid),
                    })
                } else {
                    panic!("[850BHE] The entity is not a LevelRole variant.");
                }
            }
            _ => {
                panic!("Invalid parameter combination. Only level_param should be Some, and left_outer_param and member_param should be None.");
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AstMemberFnOpeningPeriod {
    Chain,
    LvSegs(AstSegsObj),
    LvSegs_MemSegs(AstSegsObj, AstSegsObj),
}

impl AstMemberFnOpeningPeriod {
    async fn do_get_member(
        left_outer_param: Option<MultiDimensionalEntity>,
        level_param: Option<&AstSegsObj>,
        member_param: Option<&AstSegsObj>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match (left_outer_param, level_param, member_param) {
            (None, Some(lv_segs), None) => {
                let olap_obj = lv_segs.materialize(slice_tuple, context).await;
                if let MultiDimensionalEntity::LevelRole(lv_role) = olap_obj {
                    MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role: lv_role.dim_role.clone(),
                        member: meta_cache::get_member_by_gid(lv_role.level.opening_period_gid),
                    })
                } else {
                    panic!("[833BHE] The entity is not a LevelRole variant.");
                }
            }
            _ => {
                panic!("[hsb778] Invalid parameter combination. Only level_param should be Some, and left_outer_param and member_param should be None.");
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AstMemberFnCurrentMember {
    Chain,
    SegsObj(AstSegsObj),
}

impl AstMemberFnCurrentMember {
    async fn do_get_member(
        &self,
        outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        let param: MultiDimensionalEntity;

        if let Some(outer_param) = outer_param {
            param = outer_param;
        } else {
            if let AstMemberFnCurrentMember::SegsObj(ast_segs) = self {
                param = ast_segs.materialize(slice_tuple, context).await;
            } else {
                panic!("[34BH85BHE] Invalid parameter combination. Only inner_param should be Some, and outer_param should be None.")
            }
        }

        match param {
            MultiDimensionalEntity::DimensionRoleWrap(param_dim_role) => {
                for mr in slice_tuple.member_roles.iter() {
                    if let MemberRole::BaseMember { dim_role, member } = mr {
                        if dim_role.gid == param_dim_role.gid && member.level > 0 {
                            return MultiDimensionalEntity::MemberRoleWrap(
                                MemberRole::BaseMember {
                                    dim_role: param_dim_role.clone(),
                                    member: meta_cache::get_member_by_gid(member.gid),
                                },
                            );
                        }
                    }
                }
                todo!("[GGBH76] It's not implemented yet.")
            }
            _ => panic!("[34BH85BHE] The entity is not a MemberRole or a Member variant."),
        }

        // todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AstMemberFnParent {
    Chain,
    MemSegs(AstSegsObj),
}

impl AstMemberFnParent {
    async fn do_get_member(
        left_unique_param: Option<MultiDimensionalEntity>,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        if let MultiDimensionalEntity::MemberRoleWrap(mr) = left_unique_param.unwrap() {
            if let MemberRole::BaseMember { dim_role, member } = mr {
                if member.level < 1 {
                    return MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role,
                        member,
                    });
                } else {
                    let obj = context
                        .grpc_client
                        .get_universal_olap_entity_by_gid(member.parent_gid)
                        .await
                        .unwrap();
                    if let MultiDimensionalEntity::MemberWrap(member) = obj {
                        return MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                            dim_role,
                            member,
                        });
                    } else {
                        todo!()
                    }
                }
            }
        }
        todo!()
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnFirstChild {
    Chain,
    MemberSegs(AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnFirstChild {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnFirstSibling {
    Chain,
    MemberSegs(AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnFirstSibling {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnLag {
    //   "Lag" "(" <idx_exp: Expression> ")" => {
    //     AstMemberFnLag::
    Chain_IndexExp(AstExpression),
    //   },
    //   "Lag" "(" <mem_segs: Segs_Obj> "," <idx_exp: Expression> ")" => {
    //     AstMemberFnLag::
    MemberSegs_IndexExp(AstSegsObj, AstExpression),
}

impl MemberRoleAccess for AstMemberFnLag {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnLastChild {
    Chain,
    MemberSegs(AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnLastChild {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnLastSibling {
    Chain,
    MemberSegs(AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnLastSibling {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnLead {
    Chain_IndexExp(AstExpression),
    //   },
    //   "Lag" "(" <mem_segs: Segs_Obj> "," <idx_exp: Expression> ")" => {
    //     AstMemberFnLag::
    MemberSegs_IndexExp(AstSegsObj, AstExpression),
}

impl MemberRoleAccess for AstMemberFnLead {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnParallelPeriod {
    Chain,
    LevelSegs(AstSegsObj),
    LevelSegs_IndexExp(AstSegsObj, AstExpression),
    LevelSegs_IndexExp_MemberSegs(AstSegsObj, AstExpression, AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnParallelPeriod {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnPrevMember {
    Chain,
    MemberSegs(AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnPrevMember {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnNextMember {
    Chain,
    MemberSegs(AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnNextMember {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnAncestor {
    Chain_LevelSegs(AstSegsObj),
    Chain_Distance(i64),
    MemberSegs_LevelSegs(AstSegsObj, AstSegsObj),
    MemberSegs_Distance(AstSegsObj, i64),
}

impl MemberRoleAccess for AstMemberFnAncestor {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnCousin {
    Chain_AncestorMemberSegs(AstSegsObj),
    MemberSegs_AncestorMemberSegs(AstSegsObj, AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnCousin {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnDefaultMember {
    Chain,
    SegsObj(AstSegsObj),
}

impl MemberRoleAccess for AstMemberFnDefaultMember {
    fn resolve_member_role<'a>(
        &'a self,
        _slice_tuple: &'a TupleVector,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, MemberRole> {
        Box::pin(async move { todo!() })
    }
}
