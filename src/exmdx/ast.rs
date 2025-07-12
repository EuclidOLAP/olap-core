use futures::future::BoxFuture;

use crate::mdx_ast::AstSeg;
use crate::mdx_ast::Materializable;
use crate::mdx_ast::AstFormulaObject;

use crate::mdd::MultiDimensionalContext;
use crate::mdd::MultiDimensionalEntity;
use crate::exmdx::mdd::TupleVector;
use crate::mdd::GidType;
use crate::mdd::MultiDimensionalEntityLocator;

#[derive(Clone, Debug, PartialEq)]
pub struct AstSegsObj {
    pub segs: Vec<AstSeg>,
}

impl AstSegsObj {
    pub fn new(seg: AstSeg) -> Self {
        Self { segs: vec![seg] }
    }

    pub fn append(&mut self, seg: AstSeg) {
        self.segs.push(seg);
    }

    fn get_pos_gid(&self, pos: usize) -> Option<u64> {
        self.segs.get(pos)?.get_gid()
    }

    pub fn get_last_gid(&self) -> Option<u64> {
        self.get_pos_gid(self.segs.len() - 1)
    }

    pub fn get_first_gid(&self) -> Option<u64> {
        self.get_pos_gid(0)
    }
}

impl Materializable for AstSegsObj {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            let mut is_formula_member = false;

            let last_opt = self.get_last_gid();
            if let Some(last_gid) = last_opt {
                is_formula_member = GidType::entity_type(last_gid) == GidType::FormulaMember;
            }

            if is_formula_member {
                let dim_role_gid = self.get_first_gid().unwrap();
                let AstFormulaObject::CustomFormulaMember(_, exp) =
                    context.formulas_map.get(&last_opt.unwrap()).unwrap().clone();
                return MultiDimensionalEntity::FormulaMemberWrap { dim_role_gid, exp };
            }

            let ast_seg = self.segs.iter().next().unwrap();
            let head_entity: MultiDimensionalEntity =
                ast_seg.materialize(slice_tuple, context).await;

            if self.segs.len() == 1 {
                return head_entity;
            }

            match head_entity {
                MultiDimensionalEntity::DimensionRoleWrap(dim_role) => {
                    let tail_segs = AstSegsObj { segs: (self.segs[1..]).to_vec() };

                    dim_role.locate_entity(&tail_segs, slice_tuple, context).await
                }
                MultiDimensionalEntity::MemberRoleWrap(member_role) => {
                    if self.segs.len() == 1 {
                        return MultiDimensionalEntity::MemberRoleWrap(member_role);
                    }
                    todo!("[NVB676] MemberRoleWrap is not implemented yet.")
                }
                MultiDimensionalEntity::LevelRole(lv_role) => {
                    if self.segs.len() == 1 {
                        return MultiDimensionalEntity::LevelRole(lv_role);
                    }
                    todo!("[NVB666DC] MemberRoleWrap is not implemented yet.")
                }
                MultiDimensionalEntity::Cube(cube) => {
                    if self.segs.len() == 1 {
                        // return MultiDimensionalEntity::Cube(cube);
                        MultiDimensionalEntity::Cube(cube)
                    } else {
                        let tail_segs = AstSegsObj { segs: (self.segs[1..]).to_vec() };
                        cube.locate_entity(&tail_segs, slice_tuple, context).await
                    }
                }
                MultiDimensionalEntity::SetWrap(set) => {
                    let tail_segs = AstSegsObj { segs: (self.segs[1..]).to_vec() };
                    set.locate_entity(&tail_segs, slice_tuple, context).await
                }
                _ => {
                    panic!("In method AstSegsObj::materialize(): head_entity is not a DimensionRoleWrap!");
                }
            }
        })
    }
}
