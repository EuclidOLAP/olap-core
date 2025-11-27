use crate::exmdx::ast::AstSegsObj;
use crate::exmdx::ast::Materializable;
use crate::exmdx::mdd::TupleVector;
use crate::mdd::{MemberRole, MultiDimensionalEntity};
use crate::mdd::MultiDimensionalContext;
use crate::meta_cache;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnFirstSibling {
    Chain,
    MemberSegs(AstSegsObj),
}

impl AstMemberFnFirstSibling {
    pub async fn do_get_member(
        &self,
        outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        // 骨架：按照 `first_child.rs` 的风格搭建参数解析和返回结构。
        // 目前不实现具体业务逻辑，仅完成输入解析和错误处理框架。

        // 1) 解析 member role（优先使用内置 MemberSegs，否则使用 outer_param）
        let member_role: Option<MemberRole> = match self {
            AstMemberFnFirstSibling::MemberSegs(member_segs) => match member_segs
                .materialize(slice_tuple, context)
                .await
            {
                MultiDimensionalEntity::MemberRoleWrap(mr) => Some(mr),
                _ => None,
            },
            AstMemberFnFirstSibling::Chain => {
                if let Some(MultiDimensionalEntity::MemberRoleWrap(mr)) = outer_param {
                    Some(mr)
                } else {
                    None
                }
            }
        };

        if member_role.is_none() {
            panic!("[fs-001] FirstSibling requires a member (MemberSegs) or outer_param that resolves to a MemberRole");
        }

        let member_role = member_role.unwrap();

        // TODO: implement actual first-sibling logic.
        // 当前先返回解析到的 member（不改变）以完成框架搭建。
        match member_role {
            MemberRole::BaseMember { dim_role, member } => {
                MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                    dim_role,
                    member: meta_cache::get_member_by_gid(member.gid),
                })
            }
            MemberRole::FormulaMember { .. } => {
                panic!("[fs-002] FirstSibling not supported for FormulaMember");
            }
        }
    }
}
