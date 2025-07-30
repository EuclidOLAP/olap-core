use crate::exmdx::ast::AstSegsObj;

use crate::exmdx::mdd::TupleVector;

use crate::mdd::MultiDimensionalContext;
use crate::mdd::{MemberRole, MultiDimensionalEntity, Set};

use crate::exmdx::ast::Materializable;

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFunction {
    Children(AstSetFnChildren),
    BottomPercent	(	AstSetFnBottomPercent	),
    CrossJoin	(	AstSetFnCrossJoin	),
    Descendants	(	AstSetFnDescendants	),
    Except	(	AstSetFnExcept	),
    Filter	(	AstSetFnFilter	),
    Intersect	(	AstSetFnIntersect	),
    Members	(	AstSetFnMembers	),
    Order	(	AstSetFnOrder	),
    Tail	(	AstSetFnTail	),
    TopCount	(	AstSetFnTopCount	),
    TopPercent	(	AstSetFnTopPercent	),
    Union	(	AstSetFnUnion	),
    Ytd	(	AstSetFnYtd	),
    Qtd	(	AstSetFnQtd	),
    Distinct	(	AstSetFnDistinct	),
    DrilldownLevel	(	AstSetFnDrilldownLevel	),
    DrilldownLevelBottom	(	AstSetFnDrilldownLevelBottom	),
    DrillDownLevelTop	(	AstSetFnDrillDownLevelTop	),
    DrillDownMember	(	AstSetFnDrillDownMember	),
    DrillDownMemberBottom	(	AstSetFnDrillDownMemberBottom	),
    DrillDownMemberTop	(	AstSetFnDrillDownMemberTop	),
    DrillupLevel	(	AstSetFnDrillupLevel	),
    DrillupMember	(	AstSetFnDrillupMember	),
    Ancestors	(	AstSetFnAncestors	),
    BottomCount	(	AstSetFnBottomCount	),
    BottomSum	(	AstSetFnBottomSum	),
    TopSum	(	AstSetFnTopSum	),
    Extract	(	AstSetFnExtract	),
    PeriodsToDate	(	AstSetFnPeriodsToDate	),
    Generate	(	AstSetFnGenerate	),
    Head	(	AstSetFnHead	),
    Subset	(	AstSetFnSubset	),
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
            }
            _ => todo!("AstSetFunction::get_set() [HI-SHUA-927381]"),
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

#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnBottomPercent	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnCrossJoin	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDescendants	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnExcept	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnFilter	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnIntersect	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnMembers	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnOrder	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnTail	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnTopCount	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnTopPercent	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnUnion	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnYtd	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnQtd	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDistinct	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrilldownLevel	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrilldownLevelBottom	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrillDownLevelTop	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrillDownMember	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrillDownMemberBottom	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrillDownMemberTop	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrillupLevel	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnDrillupMember	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnAncestors	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnBottomCount	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnBottomSum	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnTopSum	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnExtract	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnPeriodsToDate	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnGenerate	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnHead	{ WillTodo, }
#[allow(non_camel_case_types)]	#[derive(Clone, Debug, PartialEq)]	pub enum 	AstSetFnSubset	{ WillTodo, }
