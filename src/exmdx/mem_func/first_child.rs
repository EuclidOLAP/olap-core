use crate::exmdx::ast::{AstSegsObj, Materializable};
use crate::exmdx::mdd::TupleVector;
use crate::mdd::{MemberRole, MultiDimensionalContext, MultiDimensionalEntity};
use crate::meta_cache;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnFirstChild {
    Chain,
    MemberSegs(AstSegsObj),
}

impl AstMemberFnFirstChild {
    pub async fn do_get_member(
        &self,
        outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        // 参数解析骨架（参考 PrevMember 实现风格）：
        // 1) 如果是 MemberSegs 则 materialize 并期望得到 MemberRole
        // 2) 否则尝试从 outer_param 中取出 MemberRole
        // 3) 若无法解析则 panic（带错误码）

        let member_role: Option<MemberRole> = match self {
            AstMemberFnFirstChild::MemberSegs(member_segs) => {
                match member_segs.materialize(slice_tuple, context).await {
                    MultiDimensionalEntity::MemberRoleWrap(mr) => Some(mr),
                    _ => None,
                }
            }
            AstMemberFnFirstChild::Chain => {
                if let Some(MultiDimensionalEntity::MemberRoleWrap(mr)) = outer_param {
                    Some(mr)
                } else {
                    None
                }
            }
        };

        if member_role.is_none() {
            panic!("[fc-001] FirstChild requires a Member (MemberSegs) or an outer_param resolving to MemberRole");
        }

        let member_role = member_role.unwrap();

        // 目前仅搭建结构：按类型分支返回原 member（占位）。具体的 FirstChild 逻辑稍后实现。
        match member_role {
            MemberRole::BaseMember { dim_role, member } => {
                // 返回原 member（占位实现）
                MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                    dim_role: dim_role.clone(),
                    member: meta_cache::get_member_by_gid(member.gid),
                })
            }
            MemberRole::FormulaMember { .. } => {
                panic!("[fc-002] FirstChild not supported for FormulaMember")
            }
        }
    }
}
