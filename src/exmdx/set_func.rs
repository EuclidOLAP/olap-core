use crate::exmdx::ast::AstSegsObj;

use crate::exmdx::mdd::TupleVector;

use crate::mdd::MultiDimensionalContext;
use crate::mdd::{MemberRole, MultiDimensionalEntity, Set};

use crate::exmdx::ast::Materializable;

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
