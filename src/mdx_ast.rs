use core::panic;

use crate::mdx_grammar::MdxStatementParser;
use crate::mdx_lexer::Lexer as MdxLexer;

use futures::future::BoxFuture;

use crate::exmdx::ast::{AstSegsObj, AstSet, AstTuple};

use crate::exmdx::exp_func::*;

use crate::exmdx::mdd::TupleVector;
use crate::mdd;
use crate::mdd::CellValue;
use crate::mdd::MultiDimensionalContext;
use crate::mdd::{DimensionRole, Level, LevelRole};
use crate::mdd::{MemberRole, MultiDimensionalEntity, Set};

use crate::meta_cache;

use crate::calcul::calculate;

pub trait Materializable {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut mdd::MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity>;
}

pub trait ToCellValue {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue>;
}

pub trait ToBoolValue {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool>;
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct AstCube {}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSeg {
    Gid(u64),
    Str(String),
    GidStr(u64, String),
    MemberFunc(AstMemberFunction),
    SetFunc(AstSetFunction),
    ExpFunc(AstExpFunction),
    LevelFunc(AstLevelFunction),
}

impl AstSeg {
    pub fn get_gid(&self) -> Option<u64> {
        match self {
            AstSeg::Gid(gid) => Some(*gid),
            AstSeg::GidStr(gid, _) => Some(*gid),
            _ => None,
        }
    }
}

impl Materializable for AstSeg {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut mdd::MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            match self {
                AstSeg::Gid(gid) => context.find_entity_by_gid(*gid).await,
                AstSeg::Str(seg_str) => context.find_entity_by_str(seg_str).await,
                AstSeg::GidStr(gid, _) => context.find_entity_by_gid(*gid).await,
                // MemberFunction(AstMemberFunction),
                AstSeg::MemberFunc(member_fn) => {
                    member_fn.get_member(None, slice_tuple, context).await
                }
                AstSeg::LevelFunc(lv_fn) => {
                    let lv_role = lv_fn.get_level_role(None, slice_tuple, context).await;
                    MultiDimensionalEntity::LevelRole(lv_role)
                }
                AstSeg::ExpFunc(exp_fn) => {
                    let exp_val = exp_fn.val(slice_tuple, context, None).await;
                    MultiDimensionalEntity::CellValue(exp_val)
                }
                AstSeg::SetFunc(set_fn) => {
                    let set = set_fn.get_set(None, slice_tuple, context).await;
                    MultiDimensionalEntity::SetWrap(set)
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstExpression {
    pub terms: Vec<(char, AstTerm)>,
}

impl ToCellValue for AstExpression {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            let mut result = CellValue::Invalid;
            for (index, (op, term)) in self.terms.iter().enumerate() {
                if index == 0 {
                    result = Box::pin(term.val(slice_tuple, context, None)).await;
                    continue;
                }

                let term_value = Box::pin(term.val(slice_tuple, context, None)).await;
                match *op {
                    '+' => result = result + term_value,
                    '-' => result = result - term_value,
                    _ => panic!("Invalid operator in AstExpression: {}", op),
                }
            }
            result
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstFactory {
    FactoryNum(f64),
    FactoryStr(String),
    FactorySegs(AstSegsObj),
    FactoryTuple(AstTuple),
    FactoryExp(AstExpression),
}

impl ToCellValue for AstFactory {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            match self {
                AstFactory::FactoryNum(num) => CellValue::Double(*num),
                AstFactory::FactoryStr(str) => CellValue::Str(String::from(str)),
                AstFactory::FactorySegs(segs) => match segs.materialize(slice_tuple, context).await
                {
                    MultiDimensionalEntity::MemberRoleWrap(mr) => {
                        let ovc_tp = slice_tuple.merge(&TupleVector {
                            member_roles: vec![mr],
                        });

                        let ovc = TupleVector {
                            member_roles: ovc_tp.member_roles,
                        };

                        let cell_values = calculate(vec![ovc], context).await;
                        cell_values.first().unwrap().clone()
                    }
                    MultiDimensionalEntity::FormulaMemberWrap {
                        dim_role_gid: _,
                        exp,
                    } => exp.val(slice_tuple, context, None).await,
                    // MultiDimensionalEntity::ExpFn(exp_fn) => {
                    //     exp_fn.val(slice_tuple, context, None).await
                    // }
                    MultiDimensionalEntity::CellValue(cell_value) => cell_value.clone(),
                    _ => panic!("The entity is not a CellValue variant."),
                },
                AstFactory::FactoryTuple(tuple) => {
                    match tuple.materialize(slice_tuple, context).await {
                        MultiDimensionalEntity::TupleWrap(olap_tuple) => {
                            let ovc = TupleVector {
                                member_roles: slice_tuple.merge(&olap_tuple).member_roles,
                            };
                            let cell_values = calculate(vec![ovc], context).await;
                            cell_values.first().unwrap().clone()
                        }
                        _ => panic!("The entity is not a TupleWrap variant."),
                    }
                }
                AstFactory::FactoryExp(exp) => exp.val(slice_tuple, context, None).await,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstTerm {
    pub factories: Vec<(char, AstFactory)>,
}

impl ToCellValue for AstTerm {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            let mut result = CellValue::Invalid;
            for (index, (op, factory)) in self.factories.iter().enumerate() {
                if index == 0 {
                    result = factory.val(slice_tuple, context, None).await;
                    continue;
                }

                let factory_value = factory.val(slice_tuple, context, None).await;
                match *op {
                    '*' => result = result * factory_value,
                    '/' => result = result / factory_value,
                    _ => panic!("Invalid operator in AstTerm: {}", op),
                }
            }
            result
        })
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

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFunction {
    Parent(AstMemberFnParent),
    ClosingPeriod(AstMemberFnClosingPeriod),
    OpeningPeriod(AstMemberFnOpeningPeriod),
    CurrentMember(AstMemberFnCurrentMember),
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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFunction {
    Level(AstLevelFnLevel),
    Levels(AstLevelFnLevels),
}

impl AstLevelFunction {
    pub async fn get_level_role(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> LevelRole {
        match self {
            AstLevelFunction::Level(fn_level) => {
                fn_level
                    .get_level_role(left_outer_param, slice_tuple, context)
                    .await
            }
            AstLevelFunction::Levels(fn_levels) => {
                fn_levels
                    .get_level_role(left_outer_param, slice_tuple, context)
                    .await
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFnLevel {
    Chain,
    MemSegs(AstSegsObj),
}

impl AstLevelFnLevel {
    fn do_get_level_role(&self, mr: MemberRole) -> LevelRole {
        if let MemberRole::BaseMember { dim_role, member } = mr {
            LevelRole::new(dim_role, meta_cache::get_level_by_gid(member.level_gid))
        } else {
            panic!("[003BHE] The entity is not a MemberRole variant.");
        }
    }

    async fn get_level_role(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> LevelRole {
        if let Some(MultiDimensionalEntity::MemberRoleWrap(mr)) = left_outer_param {
            return self.do_get_level_role(mr);
        }

        if let AstLevelFnLevel::MemSegs(ast_segs) = self {
            if let MultiDimensionalEntity::MemberRoleWrap(mr) =
                ast_segs.materialize(slice_tuple, context).await
            {
                return self.do_get_level_role(mr);
            }
        }

        panic!("[klu704] AstLevelFnLevel::do_get_level_role()");
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFnLevels {
    Chain_Exp(AstExpression),
    SegsObj_Exp(AstSegsObj, AstExpression),
    // dim_segs: Option<AstSegsObj>,
    // idx_exp: AstExpression,
}

impl AstLevelFnLevels {
    // pub fn new(dim_segs: Option<AstSegsObj>, idx_exp: AstExpression) -> Self {
    //     Self { dim_segs, idx_exp }
    // }

    async fn get_level_role(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> LevelRole {
        let mut param_dim_role: Option<DimensionRole> = None;
        let mut def_hierarchy_gid = 0;

        if let Some(MultiDimensionalEntity::DimensionRoleWrap(dr)) = left_outer_param {
            def_hierarchy_gid = dr.default_hierarchy_gid;
            param_dim_role = Some(dr);
        } else if let Self::SegsObj_Exp(segs_obj, _) = self {
            if let MultiDimensionalEntity::DimensionRoleWrap(dr) =
                segs_obj.materialize(slice_tuple, context).await
            {
                def_hierarchy_gid = dr.default_hierarchy_gid;
                param_dim_role = Some(dr);
            } else {
                panic!("[003BHE] The entity is not a DimensionRole variant.");
            }
        }

        if let None = param_dim_role {
            panic!("[033BHE] The entity is not a DimensionRole variant.");
        }

        let param_dim_role = param_dim_role.unwrap();

        let idx_exp = match self {
            AstLevelFnLevels::Chain_Exp(exp) => exp,
            AstLevelFnLevels::SegsObj_Exp(_, exp) => exp,
        };

        let cell_val = idx_exp.val(slice_tuple, context, None).await;
        if let CellValue::Double(idx) = cell_val {
            let lv_val = idx as u32;

            let olap_obj_level: Level = meta_cache::get_hierarchy_level(def_hierarchy_gid, lv_val);
            LevelRole::new(param_dim_role, olap_obj_level)
        } else {
            panic!("[klu704] AstLevelFnLevel::do_get_level_role()");
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFnChildren {
    Chain,
    MemSegs(AstSegsObj),
}

impl AstSetFnChildren {
    async fn do_get_set(
        left_unique_param: Option<MultiDimensionalEntity>,
        context: &mut MultiDimensionalContext,
    ) -> Set {
        if let MultiDimensionalEntity::MemberRoleWrap(mr) = left_unique_param.unwrap() {
            if let MemberRole::BaseMember { dim_role, member } = mr {
                let children = context
                    .grpc_client
                    .get_child_members_by_gid(member.gid)
                    .await
                    .unwrap();

                let tuples: Vec<TupleVector> = children
                    .into_iter()
                    .map(|child| TupleVector {
                        member_roles: vec![MemberRole::BaseMember {
                            dim_role: dim_role.clone(),
                            member: child,
                        }],
                    })
                    .collect();

                return Set { tuples };
            }
        }

        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFunction {
    Children(AstSetFnChildren),
}

impl AstSetFunction {
    pub async fn get_set(
        &self,
        left_unique_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> Set {
        match self {
            AstSetFunction::Children(AstSetFnChildren::Chain) => {
                AstSetFnChildren::do_get_set(left_unique_param, context).await
            }
            AstSetFunction::Children(AstSetFnChildren::MemSegs(segs)) => {
                let mem_role = segs.materialize(slice_tuple, context).await;
                AstSetFnChildren::do_get_set(Some(mem_role), context).await
            } // _ => todo!("AstSetFunction::get_set() [SHUA-927381]"),
        }
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolExp {
    BoolTerm(AstBoolTerm),
    NotBoolTerm(AstBoolTerm),
    BoolExpOrBoolTerm(Box<AstBoolExp>, AstBoolTerm),
}

impl ToBoolValue for AstBoolExp {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolExp::BoolTerm(bool_term) => bool_term.bool_val(slice_tuple, context).await,
                AstBoolExp::NotBoolTerm(bool_term) => {
                    !bool_term.bool_val(slice_tuple, context).await
                }
                AstBoolExp::BoolExpOrBoolTerm(bool_exp, bool_term) => {
                    let exp_bool = bool_exp.bool_val(slice_tuple, context).await;
                    if exp_bool {
                        true
                    } else {
                        bool_term.bool_val(slice_tuple, context).await
                    }
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolTerm {
    BoolFactory(AstBoolFactory),
    BoolTermAndBoolFactory(Box<AstBoolTerm>, AstBoolFactory),
}

impl ToBoolValue for AstBoolTerm {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolTerm::BoolFactory(bool_factory) => {
                    bool_factory.bool_val(slice_tuple, context).await
                }
                AstBoolTerm::BoolTermAndBoolFactory(bool_term, bool_factory) => {
                    let term_bool = bool_term.bool_val(slice_tuple, context).await;
                    if term_bool {
                        bool_factory.bool_val(slice_tuple, context).await
                    } else {
                        false
                    }
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolFactory {
    ExpressionComparesAnother(AstExpression, String, AstExpression),
    BoolExp(Box<AstBoolExp>),
    BoolFn(AstBoolFunction),
}

impl ToBoolValue for AstBoolFactory {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolFactory::ExpressionComparesAnother(exp1, op, exp2) => {
                    let val1 = exp1.val(slice_tuple, context, None).await;
                    let val2 = exp2.val(slice_tuple, context, None).await;
                    val1.logical_cmp(op, &val2)
                }
                AstBoolFactory::BoolExp(bool_exp) => bool_exp.bool_val(slice_tuple, context).await,
                AstBoolFactory::BoolFn(bool_fn) => bool_fn.bool_val(slice_tuple, context).await,
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

#[derive(Clone, Debug, PartialEq)]
pub struct AstBoolFnIsLeaf {
    pub member_segs: AstSegsObj,
}

impl AstBoolFnIsLeaf {
    pub fn new(member_segs: AstSegsObj) -> Self {
        Self { member_segs }
    }
}

impl ToBoolValue for AstBoolFnIsLeaf {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            let olap_obj = self.member_segs.materialize(slice_tuple, context).await;
            if let MultiDimensionalEntity::MemberRoleWrap(member_role) = olap_obj {
                match member_role {
                    MemberRole::BaseMember { member, .. } => member.leaf,
                    _ => true,
                }
            } else {
                panic!("[hsju6679] The entity is not a MemberRole variant.");
            }
        })
    }
}
