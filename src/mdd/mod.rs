use crate::olapmeta_grpc_client::GrpcClient;
use crate::olapmeta_grpc_client::olapmeta::UniversalOlapEntity;

enum GidType {
    Dimension,     // 100000000000001
    Hierarchy,     // 200000000000001
    Member,        // 300000000000001
    Level,         // 400000000000001
    Cube,          // 500000000000001
    DimensionRole, // 600000000000001
}

impl GidType {
    fn entity_type(gid: u64) -> GidType {
        match gid / 1_000_000_000_000_00 { // 100000000000000
            1 => GidType::Dimension,
            2 => GidType::Hierarchy,
            3 => GidType::Member,
            4 => GidType::Level,
            5 => GidType::Cube,
            6 => GidType::DimensionRole,
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
    MemberWrap(Member),
    // MemberRole(MemberRole),
    // Cube(Cube),           // 立方体实体
    // Dimension(Dimension), // 维度实体
    // Hierarchy(Hierarchy), // 层次实体
    // Level(Level),         // 层级实体
    // Member(Member),       // 成员实体
    Nothing,
}

impl MultiDimensionalEntity {
    pub fn from_universal_olap_entity(entity: &UniversalOlapEntity) -> Self {

        let entity_type = entity.olap_entity_class.as_str();

        match entity_type {
            "Member" => {
                MultiDimensionalEntity::MemberWrap(Member {
                    gid: entity.gid,
                    name: entity.name.clone(),
                })
            },
            _ => {
                panic!("Unsupported entity class: {}", entity.olap_entity_class);
            },
        }
        // MultiDimensionalEntity::Nothing
    }
}

#[derive(Debug)]
pub struct MultiDimensionalContext {
    pub cube: Cube,
    pub cube_def_tuple: Tuple, // defautl slice tuple, MDX `where statement`
    pub grpc_client: GrpcClient,
}

impl MultiDimensionalContext {
    pub async fn find_entity_by_gid(&mut self, gid: u64) -> MultiDimensionalEntity {
        println!("MultiDimensionalContext >>>>>>>>>>>>>>>>>>>>>>>> find_entity_by_gid({})", gid);
        match GidType::entity_type(gid) {
            GidType::DimensionRole => {
                let dim_role = self.grpc_client.get_dimension_role_by_gid(gid).await.unwrap();
                println!("!!!@@@###~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~dim_role: {:?}", dim_role);
                MultiDimensionalEntity::DimensionRoleWrap(dim_role)
            },
            _ => {
                panic!("Invalid gid type provided. Expected DimensionRole but found a different type.");
            }
        }
    }

    pub async fn find_entity_by_str(&mut self, seg: &String) -> MultiDimensionalEntity {
        println!("MultiDimensionalContext >>>>>>>>>>>>>>>>>>>>>>>>>>>>>> find_entity_by_str({})", seg);
        let dim_role
            = self.grpc_client.get_dimension_role_by_name(self.cube.gid, seg).await.unwrap();
        MultiDimensionalEntity::DimensionRoleWrap(dim_role)
    }
}

#[derive(Debug, Clone)]
pub struct Tuple {
    pub member_roles: Vec<MemberRole>,
}

impl Tuple {
    /*
     * self:   [Goods], [Transport], [starting region], [ending region], [starting date], [completion date], [**MeasureDimRole**]
     * other:  [Transport], [completion date], [Goods], [starting region], [ending region]
     * result: [starting date], [**MeasureDimRole**], [Transport], [completion date], [Goods], [starting region], [ending region]
     */
    pub fn merge(&self, other: &Tuple) -> Self {
        let mut result_member_roles = Vec::new();

        // 遍历 self 的 member_roles
        for ctx_mr in &self.member_roles {
            let mut found = false;
            // 检查 other 中是否有相同的 DimensionRole
            for f_mr in &other.member_roles {
                if ctx_mr.dim_role == f_mr.dim_role {
                    found = true;
                    break;
                }
            }
            // 如果没有找到相同 gid 的 DimensionRole，则添加到结果中
            if!found {
                result_member_roles.push(ctx_mr.clone());
            }
        }

        // 添加 other 的所有 member_roles 到结果中
        for f_mr in &other.member_roles {
            result_member_roles.push(f_mr.clone());
        }

        Tuple {
            member_roles: result_member_roles,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemberRole {
    pub dim_role: DimensionRole,
    pub member: Member,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DimensionRole {
    // pub gid: u64,
    // pub name: String,
    // pub cube_gid: u64,
    pub dimension_gid: u64,
}

// #[derive(Debug)]
// pub struct Dimension {}

#[derive(Debug, Clone)]
pub struct Member {
    pub gid: u64,
    pub name: String,
    // pub dimension_gid: u64,
    // pub hierarchy_gid: u64,
    // pub level_gid: u64,
    // pub level: u64,
    // pub parent_gid: u64,
}

#[derive(Debug)]
pub struct Cube {
    pub gid: u64,
    pub name: String,
}

pub struct Axis {
    pub pos_num: u32,
}
