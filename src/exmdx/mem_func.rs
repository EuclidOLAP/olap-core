use futures::future::BoxFuture;

use core::panic;

use crate::exmdx::ast::AstExpression;
use crate::exmdx::ast::AstSegsObj;

use crate::exmdx::ast::{Materializable, ToVectorValue};
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
            // ParallelPeriod( [ Level_Expression [ ,Index [ , Member_Expression ] ] ] )
            AstMemberFunction::ParallelPeriod(AstMemberFnParallelPeriod::Chain) => {
                AstMemberFnParallelPeriod::do_get_member(
                    left_outer_param,
                    None,
                    None,
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::ParallelPeriod(AstMemberFnParallelPeriod::LevelSegs(level_segs)) => {
                AstMemberFnParallelPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    None,
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::ParallelPeriod(AstMemberFnParallelPeriod::LevelSegs_IndexExp(
                level_segs,
                idx_exp,
            )) => {
                AstMemberFnParallelPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    Some(idx_exp),
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::ParallelPeriod(
                AstMemberFnParallelPeriod::LevelSegs_IndexExp_MemberSegs(
                    level_segs,
                    idx_exp,
                    member_segs,
                ),
            ) => {
                AstMemberFnParallelPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    Some(idx_exp),
                    Some(member_segs),
                    slice_tuple,
                    context,
                )
                .await
            }
            Self::PrevMember(prev_member) => {
                prev_member
                    .do_get_member(left_outer_param, slice_tuple, context)
                    .await
            },
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

impl AstMemberFnParallelPeriod {
    async fn do_get_member(
        left_outer_param: Option<MultiDimensionalEntity>,
        level_param: Option<&AstSegsObj>,
        idx_param: Option<&AstExpression>,
        member_param: Option<&AstSegsObj>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        /*
         Resolve member priority: 1) if `member_param` is present, materialize it
         and use its `MemberRole`; 2) otherwise use `left_outer_param` if it
         is a `MemberRole`; otherwise panic. After resolution print debug info;
         the actual ParallelPeriod computation is not implemented yet.
        */
        let member_role: Option<MemberRole> = if let Some(member_segs) = member_param {
            match member_segs.materialize(slice_tuple, context).await {
                MultiDimensionalEntity::MemberRoleWrap(mr) => Some(mr),
                _ => None,
            }
        } else if let Some(outer_ins_param) = left_outer_param {
            match outer_ins_param {
                MultiDimensionalEntity::MemberRoleWrap(mr) => Some(mr),
                _ => None,
            }
        } else {
            None
        };

        if member_role.is_none() {
            panic!("[pp-003] ParallelPeriod requires a member (member_param) or left_outer_param that resolves to a MemberRole");
        }

        let member_role = member_role.unwrap();


        // Resolve the LevelRole to use for ParallelPeriod:
        // 1) If `level_param` is provided, materialize it and require a LevelRole.
        // 2) Otherwise derive the LevelRole from `member_role` (must be BaseMember).
        // Panic with clear messages if neither path yields a LevelRole.
        let lv_role = if let Some(level_segs) = level_param {
            let olap_obj = level_segs.materialize(slice_tuple, context).await;
            if let MultiDimensionalEntity::LevelRole(lv_role) = olap_obj {
                lv_role
            } else {
                panic!("[pp-101] level_param did not materialize to LevelRole");
            }
        } else {
            match &member_role {
                MemberRole::BaseMember { dim_role, member } => {
                    // If possible, derive the target Level from the member's parent
                    // (the "upper" level). If the member has no parent or parent
                    // can't be fetched, fall back to the member's own level.
                    let level = if member.parent_gid != 0 {
                        let parent_obj = context
                            .grpc_client
                            .get_universal_olap_entity_by_gid(member.parent_gid)
                            .await
                            .ok();

                        if let Some(MultiDimensionalEntity::MemberWrap(parent_member)) = parent_obj {
                            meta_cache::get_level_by_gid(parent_member.level_gid)
                        } else {
                            // Fallback to member's level
                            meta_cache::get_level_by_gid(member.level_gid)
                        }
                    } else {
                        meta_cache::get_level_by_gid(member.level_gid)
                    };

                    crate::mdd::LevelRole::new(dim_role.clone(), level)
                }
                MemberRole::FormulaMember { .. } => {
                    panic!("[pp-102] Cannot derive LevelRole from a FormulaMember");
                }
            }
        };


        // Determine parallel offset: evaluate `idx_param` to an integer, default to 1.
        let offset: i64 = if let Some(idx_exp) = idx_param {
            let val = idx_exp.val(slice_tuple, context, None).await;
            match val {
                crate::mdd::VectorValue::Double(n) => n as i64,
                crate::mdd::VectorValue::Str(s) => s.parse::<i64>().unwrap_or_else(|_| {
                    panic!("[pp-201] idx_param string could not be parsed as integer: {}", s)
                }),
                crate::mdd::VectorValue::Null | crate::mdd::VectorValue::Invalid => {
                    panic!("[pp-202] idx_param evaluated to Null/Invalid")
                }
            }
        } else {
            1
        };

        // Ensure the resolved member and the target level belong to the same DimensionRole.
        // If not, panic with a clear message. Also print the target level and the
        // source member in separate, concise lines for debugging. Use the cached
        // helper `meta_cache::get_member_ancestor_on_level` to obtain the ancestor.
        let ancestor_member = match &member_role {
            MemberRole::BaseMember { dim_role, member } => {
                if dim_role.gid != lv_role.dim_role.gid {
                    panic!("[pp-400] ParallelPeriod: member and level belong to different DimensionRole (member.dim_role_gid={} vs level.dim_role_gid={})", dim_role.gid, lv_role.dim_role.gid);
                }

                if lv_role.level.level > member.level {
                    panic!("[pp-300] ParallelPeriod: target level is deeper than member's level (not supported yet)");
                }

                println!("ParallelPeriod: target level = {}", lv_role.level.level);
                println!("ParallelPeriod: source member gid = {}, member level = {}", member.gid, member.level);

                // Use cache-based traversal to get the ancestor at the target level.
                meta_cache::get_member_ancestor_on_level(member.gid, lv_role.level.gid)
            }
            MemberRole::FormulaMember { .. } => {
                panic!("[pp-401] ParallelPeriod: member is a FormulaMember (unsupported)");
            }
        };


        // 找到平行的member：使用 caching helper 或 gRPC 路径（按需）
        // Basic ParallelPeriod implementation:
        // - require a BaseMember as the source member
        // - determine the target level (from param or from the member)
        // - climb the member's parent chain until we reach the target level
        // - apply `offset` as a positional shift among siblings of that ancestor
        // Note: This implements a pragmatic, consistent behaviour; full MDX
        // semantics (e.g. when target level is deeper than source) may need
        // further refinement.

        match member_role {
            MemberRole::BaseMember { dim_role, member: source_member } => {
                // Use the ancestor obtained from cache above.
                let cur_member = ancestor_member.clone();

                // If offset == 0 simply return the ancestor at target level.
                if offset == 0 {
                    return MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role: dim_role.clone(),
                        member: meta_cache::get_member_by_gid(cur_member.gid),
                    });
                }

                // Prefer the gRPC-based helper which shifts ancestor and finds
                // the corresponding descendant by path indices. Convert offset
                // to i32 with bounds check.
                if offset < i64::from(i32::MIN) || offset > i64::from(i32::MAX) {
                    panic!("[pp-500] offset out of i32 range: {}", offset);
                }
                let offset_i32 = offset as i32;

                let shifted = meta_cache::shift_ancestor_and_find_member(
                    &mut context.grpc_client,
                    cur_member.gid,
                    source_member.gid,
                    offset_i32,
                )
                .await;

                return MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                    dim_role: dim_role.clone(),
                    member: meta_cache::get_member_by_gid(shifted.gid),
                });
            }
            MemberRole::FormulaMember { .. } => {
                panic!("[pp-306] ParallelPeriod not supported for FormulaMember");
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnPrevMember {
    Chain,
    MemberSegs(AstSegsObj),
}

impl AstMemberFnPrevMember {
    async fn do_get_member(
        &self,
        outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        // 第一步：获得 memberRole 对象
        let member_role: Option<MemberRole> = match self {
            AstMemberFnPrevMember::MemberSegs(member_segs) => {
                match member_segs.materialize(slice_tuple, context).await {
                    MultiDimensionalEntity::MemberRoleWrap(mr) => Some(mr),
                    _ => None,
                }
            }
            AstMemberFnPrevMember::Chain => {
                if let Some(MultiDimensionalEntity::MemberRoleWrap(mr)) = outer_param {
                    Some(mr)
                } else {
                    None
                }
            }
        };

        if member_role.is_none() {
            panic!("[pm-001] PrevMember requires a member (MemberSegs) or outer_param that resolves to a MemberRole");
        }

        let member_role = member_role.unwrap();

        // 第二步：找到同一 level 上的前一个成员
        match member_role {
            MemberRole::BaseMember { dim_role, member } => {
                println!("PrevMember: source member gid = {}, level_gid = {}, member level = {}", 
                    member.gid, member.level_gid, member.level);

                // 获取同一 level_gid 的所有成员（已按 gid 排序）
                let members_at_level = meta_cache::get_members_at_level(member.level_gid);

                // 找到当前成员在同层级成员中的位置
                let pos_opt = members_at_level.iter().position(|m| m.gid == member.gid);
                let pos = pos_opt.expect("[pm-102] current member not found at its level");

                if pos == 0 {
                    panic!("[pm-103] PrevMember: no previous member (index is 0 at this level)");
                } else {
                    // 取出前一个成员
                    let prev = &members_at_level[pos - 1];
                    println!("PrevMember: found previous member gid = {}", prev.gid);
                    MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role,
                        member: meta_cache::get_member_by_gid(prev.gid),
                    })
                }
            }
            MemberRole::FormulaMember { .. } => {
                panic!("[pm-104] PrevMember not supported for FormulaMember");
            }
        }
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
