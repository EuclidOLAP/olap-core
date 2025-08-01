use core::panic;

use crate::exmdx::ast::AstSegsObj;

use crate::exmdx::mdd::TupleVector;

use crate::mdd::MultiDimensionalContext;
use crate::mdd::VectorValue;
use crate::mdd::{DimensionRole, Level, LevelRole};
use crate::mdd::{MemberRole, MultiDimensionalEntity};

use crate::meta_cache;

use crate::exmdx::ast::Materializable;
use crate::exmdx::ast::{AstExpression, ToVectorValue};

#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFunction {
    Level(AstLevelFnLevel),
    Levels(AstLevelFnLevels),
    Generation(AstLevelFnGeneration),
    Generations(AstLevelFnGenerations),
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
            _ => panic!("[003BHE] The entity is not a LevelFunction variant."),
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
        if let VectorValue::Double(idx) = cell_val {
            let lv_val = idx as u32;

            let olap_obj_level: Level = meta_cache::get_hierarchy_level(def_hierarchy_gid, lv_val);
            LevelRole::new(param_dim_role, olap_obj_level)
        } else {
            panic!("[klu704] AstLevelFnLevel::do_get_level_role()");
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFnGeneration {
    Chain,
    MemberRoleSegs(AstSegsObj),
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFnGenerations {
    Chain_IndexExp(AstExpression),
    OlapObj_IndexExp(AstSegsObj, AstExpression),
}
