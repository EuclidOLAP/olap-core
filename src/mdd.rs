use core::panic;

use crate::mdx_ast::AstExpFnAvg;
use crate::mdx_ast::AstExpFnCount;
use crate::mdx_ast::AstExpFunction;
use crate::mdx_ast::{AstExpression, AstFormulaObject, AstSeg, AstSegments};

use crate::olapmeta_grpc_client::olapmeta::UniversalOlapEntity;
use crate::olapmeta_grpc_client::GrpcClient;
use std::collections::HashMap;
use std::ops;

#[derive(PartialEq)]
pub enum GidType {
    Dimension,     // 100000000000001
    Hierarchy,     // 200000000000001
    Member,        // 300000000000001
    Level,         // 400000000000001
    Cube,          // 500000000000001
    DimensionRole, // 600000000000001
    FormulaMember, // 700000000000001
}

impl GidType {
    pub fn entity_type(gid: u64) -> GidType {
        match gid / 1_000_000_000_000_00 { // 100000000000000
            1 => GidType::Dimension,
            2 => GidType::Hierarchy,
            3 => GidType::Member,
            4 => GidType::Level,
            5 => GidType::Cube,
            6 => GidType::DimensionRole,
            7 => GidType::FormulaMember,
            _ => panic!(
                "Invalid gid type: {}. Expected a gid starting with 1 (Dim), 2 (Hier), 3 (Mem), 4 (Level), 5 (Cube), or 6 (DimRole)."
                , gid),
        }
    }
}

#[derive(Debug)]
// #[derive(Clone)]
// #[derive(PartialEq)]
pub enum MultiDimensionalEntity {
    DimensionRoleWrap(DimensionRole),
    TupleWrap(Tuple),
    SetWrap(Set),
    MemberWrap(Member),
    MemberRoleWrap(MemberRole),
    FormulaMemberWrap {
        dim_role_gid: u64,
        exp: AstExpression,
    },
    ExpFn(AstExpFunction),
    // Cube(Cube),           // 立方体实体
    // Dimension(Dimension), // 维度实体
    // Hierarchy(Hierarchy), // 层次实体
    // Level(Level),         // 层级实体
    Nothing,
}

#[derive(Debug, Clone)]
pub enum CellValue {
    Double(f64),
    Str(String),
    Null,
    Invalid,
}

// CellValue + CellValue
impl ops::Add for CellValue {
    type Output = CellValue;

    fn add(self, other: CellValue) -> CellValue {
        match (self, other) {
            (CellValue::Double(num_1), CellValue::Double(num_2)) => {
                CellValue::Double(num_1 + num_2)
            }
            (CellValue::Double(num_1), CellValue::Str(str_2)) => {
                CellValue::Str(format!("{}{}", num_1, str_2))
            }
            (CellValue::Str(str_1), CellValue::Double(num_2)) => {
                CellValue::Str(format!("{}{}", str_1, num_2))
            }
            (CellValue::Str(str_1), CellValue::Str(str_2)) => {
                CellValue::Str(format!("{}{}", str_1, str_2))
            }
            _ => CellValue::Invalid,
        }
    }
}

// CellValue - CellValue
impl ops::Sub for CellValue {
    type Output = CellValue;

    fn sub(self, other: CellValue) -> CellValue {
        match (self, other) {
            (CellValue::Double(num_1), CellValue::Double(num_2)) => {
                CellValue::Double(num_1 - num_2)
            }
            _ => CellValue::Invalid,
        }
    }
}

// CellValue * CellValue
impl ops::Mul for CellValue {
    type Output = CellValue;

    fn mul(self, other: CellValue) -> CellValue {
        match (self, other) {
            (CellValue::Double(num_1), CellValue::Double(num_2)) => {
                CellValue::Double(num_1 * num_2)
            }
            _ => CellValue::Invalid,
        }
    }
}

// CellValue / CellValue
impl ops::Div for CellValue {
    type Output = CellValue;

    fn div(self, other: CellValue) -> CellValue {
        match (self, other) {
            (CellValue::Double(num_1), CellValue::Double(num_2)) => {
                if num_2 == 0.0 {
                    CellValue::Invalid
                } else {
                    CellValue::Double(num_1 / num_2)
                }
            }
            _ => CellValue::Invalid,
        }
    }
}

impl MultiDimensionalEntity {
    pub fn from_universal_olap_entity(entity: &UniversalOlapEntity) -> Self {
        let entity_type = entity.olap_entity_class.as_str();

        match entity_type {
            "Member" => MultiDimensionalEntity::MemberWrap(Member {
                gid: entity.gid,
                name: entity.name.clone(),
                level: entity.level,
                measure_index: entity.measure_index,
                parent_gid: entity.parent_gid,
            }),
            _ => {
                panic!("Unsupported entity class: {}", entity.olap_entity_class);
            }
        }
        // MultiDimensionalEntity::Nothing
    }
}

pub trait MultiDimensionalEntityLocator {
    async fn locate_entity(
        &self,
        segs: &AstSegments,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity;

    async fn locate_entity_by_gid(
        &self,
        gid: u64,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity;

    async fn locate_entity_by_seg(
        &self,
        seg: &String,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity;
}

#[derive(Debug)]
pub struct MultiDimensionalContext {
    pub cube: Cube,
    pub cube_def_tuple: Tuple,
    pub where_tuple: Option<Tuple>,
    pub query_slice_tuple: Tuple,
    pub grpc_client: GrpcClient,
    pub formulas_map: HashMap<u64, AstFormulaObject>,
}

impl MultiDimensionalContext {
    pub async fn find_entity_by_gid(&mut self, gid: u64) -> MultiDimensionalEntity {
        match GidType::entity_type(gid) {
            GidType::DimensionRole => {
                let dim_role = self
                    .grpc_client
                    .get_dimension_role_by_gid(gid)
                    .await
                    .unwrap();
                MultiDimensionalEntity::DimensionRoleWrap(dim_role)
            }
            _ => {
                panic!(
                    "Invalid gid type provided. Expected DimensionRole but found a different type."
                );
            }
        }
    }

    pub async fn find_entity_by_str(&mut self, seg: &String) -> MultiDimensionalEntity {
        println!(
            "MultiDimensionalContext >>>>>>>>>>>>>>>>>>>>>>>>>>>>>> find_entity_by_str({})",
            seg
        );
        let dim_role = self
            .grpc_client
            .get_dimension_role_by_name(self.cube.gid, seg)
            .await
            .unwrap();
        MultiDimensionalEntity::DimensionRoleWrap(dim_role)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub member_roles: Vec<MemberRole>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Set {
    pub tuples: Vec<Tuple>,
}

impl MultiDimensionalEntityLocator for Set {
    async fn locate_entity(
        &self,
        segs: &AstSegments,
        _slice_tuple: &Tuple,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        let seg_list = &segs.segs;
        
        let seg = seg_list.iter().next().unwrap();

        match seg {
            AstSeg::ExpFn(exp_fn) => match exp_fn {
                AstExpFunction::Avg(_) => {
                    if seg_list.len() > 1 {
                        panic!("Avg function can only have one segment. hsbt2839");
                    }
                    let set_copy = self.clone();
                    let avg_fn = AstExpFnAvg::OuterParam(set_copy);
                    return MultiDimensionalEntity::ExpFn(AstExpFunction::Avg(avg_fn));
                }
                AstExpFunction::Count(_) => {
                    if seg_list.len() > 1 {
                        panic!("Count function can only have one segment. hs8533BJ");
                    }
                    let set_copy = self.clone();
                    let count_fn = AstExpFnCount::OuterParam(set_copy);
                    return MultiDimensionalEntity::ExpFn(AstExpFunction::Count(count_fn));
                }
            },
            _ => panic!("The entity is not a Gid or a Str variant. 3"),
        }
    }

    async fn locate_entity_by_gid(
        &self,
        _gid: u64,
        _slice_tuple: &Tuple,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!()
    }

    async fn locate_entity_by_seg(
        &self,
        _seg: &String,
        _slice_tuple: &Tuple,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!()
    }
}

impl Tuple {
    /*
     * self:   [Goods], [Transport], [starting region], [ending region], [starting date], [completion date], [**MeasureDimRole**]
     * other:  [Transport], [completion date], [Goods], [starting region], [ending region]
     * result: [starting date], [**MeasureDimRole**], [Transport], [completion date], [Goods], [starting region], [ending region]
     */
    pub fn merge(&self, other: &Tuple) -> Self {
        let mut mrs = self.member_roles.clone();
        mrs.retain(|mr| {
            !other
                .member_roles
                .iter()
                .any(|or| or.get_dim_role_gid() == mr.get_dim_role_gid())
        });
        mrs.extend(other.member_roles.clone());

        Tuple { member_roles: mrs }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberRole {
    BaseMember {
        dim_role: DimensionRole,
        member: Member,
    },
    FormulaMember {
        dim_role_gid: u64,
        exp: AstExpression,
    },
}

impl MemberRole {
    pub fn get_dim_role_gid(&self) -> u64 {
        match self {
            MemberRole::BaseMember { dim_role, .. } => dim_role.gid,
            MemberRole::FormulaMember {
                dim_role_gid,
                exp: _,
            } => *dim_role_gid,
        }
    }
}

impl MultiDimensionalEntityLocator for MemberRole {
    async fn locate_entity(
        &self,
        segs: &AstSegments,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        let seg_list = &segs.segs;
        
        let seg = seg_list.first().unwrap();
        match seg {
            AstSeg::MemberFunction(member_fn) => {
                member_fn
                    .get_member(
                        Some(MultiDimensionalEntity::MemberRoleWrap(self.clone())),
                        slice_tuple,
                        context,
                    )
                    .await
            }
            AstSeg::SetFunction(set_fn) => {
                let set = set_fn
                    .get_set(
                        Some(MultiDimensionalEntity::MemberRoleWrap(self.clone())),
                        context,
                    )
                    .await;

                if seg_list.len() == 1 {
                    MultiDimensionalEntity::SetWrap(set)
                } else {
                    // let tail_segs = AstSegments::Segs(seg_list[1..].to_vec());
                    let tail_segs = AstSegments{
                        segs: (seg_list[1..].to_vec())
                    };
                    set.locate_entity(&tail_segs, slice_tuple, context).await
                }
            }
            _ => panic!("Panic in MemberRole::locate_entity() .. 67HUSran .."),
        }
    }

    async fn locate_entity_by_gid(
        &self,
        _gid: u64,
        _slice_tuple: &Tuple,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!("MemberRole::locate_entity_by_gid() not implemented yet.")
    }

    async fn locate_entity_by_seg(
        &self,
        _seg: &String,
        _slice_tuple: &Tuple,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!("MemberRole::locate_entity_by_seg() not implemented yet.")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DimensionRole {
    pub gid: u64,
    // pub name: String,
    // pub cube_gid: u64,
    pub dimension_gid: u64,
    pub measure_flag: bool,
}

impl MultiDimensionalEntityLocator for DimensionRole {
    async fn locate_entity(
        &self,
        segs: &AstSegments,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        let seg_list = &segs.segs;
        
        let seg = seg_list.iter().next().unwrap();
        let entity = match seg {
            AstSeg::Gid(gid) | AstSeg::GidStr(gid, _) => {
                self.locate_entity_by_gid(*gid, slice_tuple, context).await
            }
            AstSeg::Str(seg) => self.locate_entity_by_seg(seg, slice_tuple, context).await,
            _ => panic!("The entity is not a Gid or a Str variant. 3"),
        };

        match entity {
            MultiDimensionalEntity::MemberRoleWrap(member_role) => {
                if seg_list.len() == 1 {
                    return MultiDimensionalEntity::MemberRoleWrap(member_role);
                }

                let tail_segs = AstSegments{
                    segs: (seg_list[1..].to_vec())
                };
                member_role
                    .locate_entity(&tail_segs, slice_tuple, context)
                    .await
            }
            _ => {
                panic!("[DimRole] locate_entity() Unsupported entity class.");
            }
        }
    }

    async fn locate_entity_by_gid(
        &self,
        gid: u64,
        _slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        // let dim_gid = self.dimension_gid;
        let olap_entity = context
            .grpc_client
            .locate_universal_olap_entity_by_gid(self.gid, gid)
            .await
            .unwrap();

        match olap_entity {
            MultiDimensionalEntity::MemberWrap(member) => {
                let member_role = MemberRole::BaseMember {
                    dim_role: self.clone(),
                    member,
                };
                // return MultiDimensionalEntity::MemberRoleWrap(member_role);
                MultiDimensionalEntity::MemberRoleWrap(member_role)
            }
            _ => {
                panic!("[DimRole] locate_entity_by_gid() Unsupported entity class.");
            }
        }
    }

    async fn locate_entity_by_seg(
        &self,
        _seg: &String,
        _slice_tuple: &Tuple,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!("DimensionRole::locate_entity_by_seg() not implemented yet.");
    }
}

// #[derive(Debug)]
// pub struct Dimension {}

#[derive(Debug, Clone, PartialEq)]
pub struct Member {
    pub gid: u64,
    pub name: String,
    // pub dimension_gid: u64,
    // pub hierarchy_gid: u64,
    // pub level_gid: u64,
    pub level: u32,
    pub parent_gid: u64,
    pub measure_index: u32,
}

#[derive(Debug)]
pub struct Cube {
    pub gid: u64,
    pub name: String,
}

#[derive(Debug)]
pub struct Axis {
    pub set: Set,
    pub pos_num: u32,
}

impl Axis {
    pub fn axis_vec_cartesian_product(
        axes: &Vec<Axis>,
        context: &MultiDimensionalContext,
    ) -> Vec<OlapVectorCoordinate> {
        let count = axes.len();

        if count == 0 {
            panic!("Axis::axis_vec_cartesian_product() axes is empty.");
        }

        if count == 1 {
            let mut ov_coordinates: Vec<OlapVectorCoordinate> = Vec::new();
            let axis = axes.iter().next().unwrap();
            for ax_tuple in &axis.set.tuples {
                ov_coordinates.push(OlapVectorCoordinate {
                    member_roles: ax_tuple.member_roles.clone(),
                });
            }
            return ov_coordinates;
        }

        let mut axes_itor = axes.iter();
        let axis_left = axes_itor.next().unwrap();
        let mut finished_tuples: Vec<Tuple> = Vec::new();
        for ax_tuple in &axis_left.set.tuples {
            finished_tuples.push(ax_tuple.clone());
        }

        let mut transitional_tuples: Vec<Tuple>;

        for axis_right in axes_itor {
            transitional_tuples = Vec::new();

            for tuple in finished_tuples.iter() {
                for rig_tuple in axis_right.set.tuples.iter() {
                    let merged_tuple = tuple.merge(rig_tuple);
                    transitional_tuples.push(merged_tuple);
                }
            }

            finished_tuples = transitional_tuples;
        }

        let mut ov_coordinates: Vec<OlapVectorCoordinate> = Vec::new();
        for tuple in finished_tuples {
            ov_coordinates.push(OlapVectorCoordinate {
                member_roles: context.query_slice_tuple.merge(&tuple).member_roles,
            });
        }

        ov_coordinates
    }
}

#[derive(Debug)]
pub struct OlapVectorCoordinate {
    pub member_roles: Vec<MemberRole>,
}
