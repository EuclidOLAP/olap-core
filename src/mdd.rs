use core::panic;

use crate::mdx_ast::ToCellValue;
use crate::meta_cache;

// use crate::mdx_ast::AstNumFnAvg;
// use crate::mdx_ast::AstNumFnCount;
use crate::mdx_ast::AstExpFunction;
use crate::mdx_ast::{AstExpression, AstSeg};

use crate::exmdx::ast::{AstCustomObject, AstSegsObj};
use crate::exmdx::mdd::TupleVector;

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

#[derive(Debug, Clone, PartialEq)]
pub enum MultiDimensionalEntity {
    Level(Level),
    LevelRole(LevelRole),
    DimensionRoleWrap(DimensionRole),
    TupleWrap(TupleVector),
    SetWrap(Set),
    MemberWrap(Member),
    MemberRoleWrap(MemberRole),
    FormulaMemberWrap {
        dim_role_gid: u64,
        exp: AstExpression,
    },
    ExpFn(AstExpFunction),
    CellValue(CellValue),
    Cube(Cube),
    // Dimension(Dimension), // 维度实体
    // Hierarchy(Hierarchy), // 层次实体
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
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

impl CellValue {
    pub fn logical_cmp(&self, op: &String, other: &CellValue) -> bool {
        match (self, other) {
            (CellValue::Double(a), CellValue::Double(b)) => match op.as_str() {
                "<" => a < b,
                "<=" => a <= b,
                "=" => a == b,
                "<>" => a != b,
                ">" => a > b,
                ">=" => a >= b,
                _ => false,
            },
            (CellValue::Str(a), CellValue::Str(b)) => match op.as_str() {
                "<" => a < b,
                "<=" => a <= b,
                "=" => a == b,
                "<>" => a != b,
                ">" => a > b,
                ">=" => a >= b,
                _ => false,
            },
            _ => false,
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
                level_gid: entity.level_gid,
                measure_index: entity.measure_index,
                parent_gid: entity.parent_gid,
                leaf: entity.leaf,
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
        segs: &AstSegsObj,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity;

    async fn locate_entity_by_gid(
        &self,
        gid: u64,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity;

    async fn locate_entity_by_seg(
        &self,
        seg: &String,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity;
}

#[derive(Debug)]
pub struct MultiDimensionalContext {
    pub cube: Cube,
    // pub cube_def_tuple: Tuple,
    // pub where_tuple: Option<Tuple>,
    pub query_slice_tuple: TupleVector,
    pub grpc_client: GrpcClient,
    pub formulas_map: HashMap<u64, AstCustomObject>,
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
            GidType::Cube => {
                let cube = meta_cache::get_cube_by_gid(gid);
                MultiDimensionalEntity::Cube(cube)
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
pub struct Set {
    pub tuples: Vec<TupleVector>,
}

impl MultiDimensionalEntityLocator for Set {
    async fn locate_entity(
        &self,
        segs: &AstSegsObj,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        let seg_list = &segs.segs;

        let seg = seg_list.iter().next().unwrap();

        match seg {
            AstSeg::ExpFunc(exp_fn) => match exp_fn {
                AstExpFunction::Avg(avg_fn) => {
                    if seg_list.len() > 1 {
                        panic!("Avg function can only have one segment. hsbt2839");
                    }

                    let fn_result = avg_fn.val(slice_tuple, context, Some(MultiDimensionalEntity::SetWrap(self.clone()))).await;
                    MultiDimensionalEntity::CellValue(fn_result)

                    // // let set_copy = self.clone();
                    // // let avg_fn = AstNumFnAvg::OuterParam(set_copy);
                    // let outter_param = MultiDimensionalEntity::SetWrap(self.clone());
                    // return MultiDimensionalEntity::ExpFnWithOutterParam(outter_param, avg_fn.clone());
                }
                AstExpFunction::Count(count_fn) => {
                    if seg_list.len() > 1 {
                        panic!("Count function can only have one segment. hs8533BJ");
                    }

                    let fn_result = count_fn.val(slice_tuple, context, Some(MultiDimensionalEntity::SetWrap(self.clone()))).await;
                    MultiDimensionalEntity::CellValue(fn_result)

                    // // let set_copy = self.clone();
                    // // let count_fn = AstNumFnCount::OuterParam(set_copy);
                    // let outter_param = MultiDimensionalEntity::SetWrap(self.clone());
                    // return MultiDimensionalEntity::ExpFnWithOutterParam(outter_param, count_fn.clone());
                }
                AstExpFunction::Sum(exp_fn_sum) => {
                    if seg_list.len() > 1 {
                        panic!("Avg function can only have one segment. hsbt2839rr");
                    }
                    let value = exp_fn_sum
                        .val(
                            slice_tuple,
                            context,
                            Some(MultiDimensionalEntity::SetWrap(self.clone())),
                        )
                        .await;
                    MultiDimensionalEntity::CellValue(value)
                }
                AstExpFunction::Max(exp_fn_max) => {
                    if seg_list.len() > 1 {
                        panic!("Avg function can only have one segment. hsbt2839tt");
                    }
                    let value = exp_fn_max
                        .val(
                            slice_tuple,
                            context,
                            Some(MultiDimensionalEntity::SetWrap(self.clone())),
                        )
                        .await;
                    MultiDimensionalEntity::CellValue(value)
                }
                AstExpFunction::Min(exp_fn_min) => {
                    if seg_list.len() > 1 {
                        panic!("Avg function can only have one segment. hsbt2839yy");
                    }
                    let value = exp_fn_min
                        .val(
                            slice_tuple,
                            context,
                            Some(MultiDimensionalEntity::SetWrap(self.clone())),
                        )
                        .await;
                    MultiDimensionalEntity::CellValue(value)
                }
                _ => {
                    todo!("[bhsHC957] Set::locate_entity() Unsupported ExpFn function.")
                }
            },
            _ => panic!("The entity is not a Gid or a Str variant. 3"),
        }
    }

    async fn locate_entity_by_gid(
        &self,
        _gid: u64,
        _slice_tuple: &TupleVector,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!()
    }

    async fn locate_entity_by_seg(
        &self,
        _seg: &String,
        _slice_tuple: &TupleVector,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelRole {
    pub dim_role: DimensionRole,
    pub level: Level,
}

impl LevelRole {
    pub fn new(dim_role: DimensionRole, level: Level) -> Self {
        LevelRole { dim_role, level }
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
        segs: &AstSegsObj,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        let seg_list = &segs.segs;

        let seg = seg_list.first().unwrap();
        match seg {
            AstSeg::MemberFunc(member_fn) => {
                member_fn
                    .get_member(
                        Some(MultiDimensionalEntity::MemberRoleWrap(self.clone())),
                        slice_tuple,
                        context,
                    )
                    .await
            }
            AstSeg::SetFunc(set_fn) => {
                let set = set_fn
                    .get_set(
                        Some(MultiDimensionalEntity::MemberRoleWrap(self.clone())),
                        slice_tuple,
                        context,
                    )
                    .await;

                if seg_list.len() == 1 {
                    MultiDimensionalEntity::SetWrap(set)
                } else {
                    // let tail_segs = AstSegments::Segs(seg_list[1..].to_vec());
                    let tail_segs = AstSegsObj {
                        segs: (seg_list[1..].to_vec()),
                    };
                    set.locate_entity(&tail_segs, slice_tuple, context).await
                }
            }
            AstSeg::LevelFunc(level_fn) => {
                if seg_list.len() > 1 {
                    todo!("[bhso9957] MemberRole::locate_entity() LevelFn not implemented yet.");
                }
                let lv_role: LevelRole = level_fn
                    .get_level_role(
                        Some(MultiDimensionalEntity::MemberRoleWrap(self.clone())),
                        slice_tuple,
                        context,
                    )
                    .await;
                MultiDimensionalEntity::LevelRole(lv_role)
            }
            AstSeg::ExpFunc(exp_fn) => {
                if seg_list.len() > 1 {
                    todo!("[bhso9957] MemberRole::locate_entity() LevelFn not implemented yet.");
                }
                let cell_val = exp_fn
                    .val(
                        slice_tuple,
                        context,
                        Some(MultiDimensionalEntity::MemberRoleWrap(self.clone())),
                    )
                    .await;
                MultiDimensionalEntity::CellValue(cell_val)
            }
            _ => panic!("Panic in MemberRole::locate_entity() .. 67HUSran .."),
        }
    }

    async fn locate_entity_by_gid(
        &self,
        _gid: u64,
        _slice_tuple: &TupleVector,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!("MemberRole::locate_entity_by_gid() not implemented yet.")
    }

    async fn locate_entity_by_seg(
        &self,
        _seg: &String,
        _slice_tuple: &TupleVector,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!("MemberRole::locate_entity_by_seg() not implemented yet.")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub gid: u64,
    pub name: String,
    pub level: u32,
    pub dimension_gid: u64,
    pub hierarchy_gid: u64,
    pub opening_period_gid: u64,
    pub closing_period_gid: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DimensionRole {
    pub gid: u64,
    // pub name: String,
    // pub cube_gid: u64,
    pub dimension_gid: u64,
    pub default_hierarchy_gid: u64,
    pub measure_flag: bool,
}

impl MultiDimensionalEntityLocator for DimensionRole {
    async fn locate_entity(
        &self,
        segs: &AstSegsObj,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        let seg_list = &segs.segs;

        let seg = seg_list.iter().next().unwrap();
        let entity = match seg {
            AstSeg::Gid(gid) | AstSeg::GidStr(gid, _) => {
                self.locate_entity_by_gid(*gid, slice_tuple, context).await
            }
            AstSeg::Str(seg) => self.locate_entity_by_seg(seg, slice_tuple, context).await,
            AstSeg::LevelFunc(level_fn) => MultiDimensionalEntity::LevelRole(
                level_fn
                    .get_level_role(
                        Some(MultiDimensionalEntity::DimensionRoleWrap(self.clone())),
                        slice_tuple,
                        context,
                    )
                    .await,
            ),
            AstSeg::MemberFunc(member_fn) => {
                member_fn
                    .get_member(
                        Some(MultiDimensionalEntity::DimensionRoleWrap(self.clone())),
                        slice_tuple,
                        context,
                    )
                    .await
            }
            _ => panic!("The entity is not a Gid or a Str variant. 3"),
        };

        match entity {
            MultiDimensionalEntity::MemberRoleWrap(member_role) => {
                if seg_list.len() == 1 {
                    return MultiDimensionalEntity::MemberRoleWrap(member_role);
                }

                let tail_segs = AstSegsObj {
                    segs: (seg_list[1..].to_vec()),
                };
                member_role
                    .locate_entity(&tail_segs, slice_tuple, context)
                    .await
            }
            MultiDimensionalEntity::LevelRole(lv_role) => {
                if seg_list.len() == 1 {
                    return MultiDimensionalEntity::LevelRole(lv_role);
                }
                todo!("[HDJS8840] DimensionRole::locate_entity() LevelRole not implemented yet.")
            }
            _ => {
                panic!("[DimRole] locate_entity() Unsupported entity class.");
            }
        }
    }

    async fn locate_entity_by_gid(
        &self,
        gid: u64,
        _slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match GidType::entity_type(gid) {
            GidType::Member => {
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
            GidType::Level => {
                let level = meta_cache::get_level_by_gid(gid);
                MultiDimensionalEntity::LevelRole(LevelRole::new(self.clone(), level))
            }
            _ => {
                todo!("Unsupported entity type.");
            }
        }
    }

    async fn locate_entity_by_seg(
        &self,
        _seg: &String,
        _slice_tuple: &TupleVector,
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
    pub level_gid: u64,
    pub level: u32,
    pub parent_gid: u64,
    pub measure_index: u32,
    pub leaf: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    pub gid: u64,
    pub name: String,
}

impl MultiDimensionalEntityLocator for Cube {
    async fn locate_entity(
        &self,
        segs: &AstSegsObj,
        _slice_tuple: &TupleVector,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        if segs.segs.len() != 1 {
            todo!("[bhs987sub3] Cube::locate_entity() not implemented yet.");
        }

        let seg = segs.segs.first().unwrap();

        if let AstSeg::ExpFunc(AstExpFunction::LookupCube(look_up_fn)) = seg {
            let mut look_up_fn = look_up_fn.clone();
            look_up_fn.set_cube(self.clone());
            MultiDimensionalEntity::ExpFn(AstExpFunction::LookupCube(look_up_fn))
        } else {
            unimplemented!("[nsbk8562] Cube::locate_entity() not implemented yet.")
        }
    }

    async fn locate_entity_by_gid(
        &self,
        _gid: u64,
        _slice_tuple: &TupleVector,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!()
    }

    async fn locate_entity_by_seg(
        &self,
        _seg: &String,
        _slice_tuple: &TupleVector,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        todo!()
    }
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
    ) -> Vec<TupleVector> {
        let count = axes.len();

        if count == 0 {
            panic!("Axis::axis_vec_cartesian_product() axes is empty.");
        }

        if count == 1 {
            let mut ov_coordinates: Vec<TupleVector> = Vec::new();
            let axis = axes.iter().next().unwrap();
            for ax_tuple in &axis.set.tuples {
                ov_coordinates.push(TupleVector {
                    member_roles: ax_tuple.member_roles.clone(),
                });
            }
            return ov_coordinates;
        }

        let mut axes_itor = axes.iter();
        let axis_left = axes_itor.next().unwrap();
        let mut finished_tuples: Vec<TupleVector> = Vec::new();
        for ax_tuple in &axis_left.set.tuples {
            finished_tuples.push(ax_tuple.clone());
        }

        let mut transitional_tuples: Vec<TupleVector>;

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

        let mut ov_coordinates: Vec<TupleVector> = Vec::new();
        for tuple in finished_tuples {
            ov_coordinates.push(TupleVector {
                member_roles: context.query_slice_tuple.merge(&tuple).member_roles,
            });
        }

        ov_coordinates
    }
}
