// src/olapmeta_grpc_client.rs

use crate::cfg;
use tokio::time::{sleep, Duration};

use olapmeta::olap_meta_service_client::OlapMetaServiceClient;
use olapmeta::EmptyParameterRequest;
use olapmeta::GetChildMembersByGidRequest;
use olapmeta::GetDefaultDimensionMemberRequest;
use olapmeta::GetDimensionRoleByGidRequest;
use olapmeta::GetDimensionRolesByCubeGidRequest;
use olapmeta::GetUniversalOlapEntityByGidRequest;
use olapmeta::GrpcMember;
use olapmeta::LocateOlapEntityRequest;
use olapmeta::UniversalOlapEntity;
use olapmeta::{CubeGidRequest, CubeMetaResponse, CubeNameRequest};
use std::fmt;
use tonic::{transport::Channel, Request};

use crate::mdd;
use crate::mdd::MultiDimensionalEntity;

pub mod olapmeta {
    tonic::include_proto!("olapmeta");
}

pub struct GrpcClient {
    client: OlapMetaServiceClient<Channel>,
}

fn grpc_to_olap_member(grpc_member: GrpcMember) -> mdd::Member {
    mdd::Member {
        gid: grpc_member.gid,
        name: grpc_member.name,
        // dimension_gid: grpc_member.dimension_gid,
        // hierarchy_gid: grpc_member.hierarchy_gid,
        level_gid: grpc_member.level_gid,
        level: grpc_member.level,
        parent_gid: grpc_member.parent_gid,
        measure_index: grpc_member.measure_index,
        leaf: grpc_member.leaf,
    }
}

impl GrpcClient {
    pub async fn get_cli() -> Self {
        let config = cfg::get_cfg();
        loop {
            let res = GrpcClient::new(config.meta_grpc_url.clone()).await;
            match res {
                Ok(client) => {
                    return client;
                }
                Err(e) => {
                    println!("Failed to connect to gRPC server: {}. Retrying in 3 seconds...", e);
                }
            }
            sleep(Duration::from_secs(3)).await;
        }
    }

    // 创建新的客户端实例
    pub async fn new(address: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = OlapMetaServiceClient::connect(address).await?;
        Ok(GrpcClient { client })
    }

    // 通过 GID 获取 Cube
    pub async fn get_cube_by_gid(
        &mut self,
        gid: u64,
    ) -> Result<CubeMetaResponse, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> get_cube_by_gid({})", gid);
        let request = Request::new(CubeGidRequest { gid });
        let response = self.client.get_cube_by_gid(request).await?;
        Ok(response.into_inner())
    }

    // 通过 Name 获取 Cube
    pub async fn get_cube_by_name(
        &mut self,
        name: String,
    ) -> Result<CubeMetaResponse, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> get_cube_by_name({})", name);
        let request = Request::new(CubeNameRequest { name });
        let response = self.client.get_cube_by_name(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_dimension_roles_by_cube_gid(
        &mut self,
        cube_gid: u64,
    ) -> Result<Vec<mdd::DimensionRole>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get_dimension_roles_by_cube_gid(GetDimensionRolesByCubeGidRequest { gid: cube_gid })
            .await?;

        // 将响应数据解析为 DimensionRole 列表
        let dimension_roles: Vec<mdd::DimensionRole> = response
            .into_inner()
            .dimension_roles
            .into_iter()
            .map(|grpc_dr| mdd::DimensionRole {
                gid: grpc_dr.gid,
                // name: grpc_dr.name,
                // cube_gid: grpc_dr.cube_gid,
                dimension_gid: grpc_dr.dimension_gid,
                default_hierarchy_gid: grpc_dr.default_hierarchy_gid,
                measure_flag: grpc_dr.measure_flag == 1,
            })
            .collect();

        Ok(dimension_roles)
    }

    pub async fn get_default_dimension_member_by_dimension_gid(
        &mut self,
        dimension_gid: u64,
    ) -> Result<mdd::Member, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> get_default_dimension_member_by_dimension_gid({})", dimension_gid);
        let request = GetDefaultDimensionMemberRequest { dimension_gid };

        let response = self.client.get_default_dimension_member_by_dimension_gid(request).await?;

        let grpc_member = response.into_inner();

        Ok(grpc_to_olap_member(grpc_member))
    }

    pub async fn get_child_members_by_gid(
        &mut self,
        parent_member_gid: u64,
    ) -> Result<Vec<mdd::Member>, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> get_child_members_by_gid({})", parent_member_gid);
        let req = GetChildMembersByGidRequest { parent_member_gid };

        let response = self.client.get_child_members_by_gid(req).await?;
        let response = response.into_inner();
        let children = response
            .child_members
            .into_iter()
            .map(|grpc_member| grpc_to_olap_member(grpc_member))
            .collect();

        Ok(children)
    }

    pub async fn get_dimension_role_by_gid(
        &mut self,
        dim_role_gid: u64,
    ) -> Result<mdd::DimensionRole, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> get_dimension_role_by_gid({})", dim_role_gid);
        let req = GetDimensionRoleByGidRequest { dimension_role_gid: dim_role_gid };

        let response = self.client.get_dimension_role_by_gid(req).await?;

        let grpc_dim_role = response.into_inner();

        let dim_role = mdd::DimensionRole {
            gid: grpc_dim_role.gid,
            dimension_gid: grpc_dim_role.dimension_gid,
            default_hierarchy_gid: grpc_dim_role.default_hierarchy_gid,
            measure_flag: grpc_dim_role.measure_flag == 1,
        };

        Ok(dim_role)
    }

    pub async fn get_dimension_role_by_name(
        &mut self,
        cube_gid: u64,
        dimension_role_name: &String,
    ) -> Result<mdd::DimensionRole, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> get_dimension_role_by_name({}, {})",cube_gid, dimension_role_name);
        let request = Request::new(olapmeta::GetDimensionRoleByNameRequest {
            cube_gid,
            dimension_role_name: dimension_role_name.clone(),
        });

        // 调用 gRPC 服务
        let response = self.client.get_dimension_role_by_name(request).await?;

        // 提取响应数据
        let grpc_dim_role = response.into_inner();

        // 将 grpc response 转换为 mdd::DimensionRole
        let dim_role = mdd::DimensionRole {
            gid: grpc_dim_role.gid,
            dimension_gid: grpc_dim_role.dimension_gid,
            default_hierarchy_gid: grpc_dim_role.default_hierarchy_gid,
            measure_flag: grpc_dim_role.measure_flag == 1,
        };

        Ok(dim_role)
    }

    pub async fn locate_universal_olap_entity_by_gid(
        &mut self,
        origin_gid: u64,
        target_entity_gid: u64,
    ) -> Result<MultiDimensionalEntity, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> locate_universal_olap_entity_by_gid({}, {})", origin_gid, target_entity_gid );
        let request = Request::new(LocateOlapEntityRequest {
            origin_gid,
            target_entity_gid,
            target_entity_name: "".to_string(),
        });

        let universal_olap_entity =
            self.client.locate_universal_olap_entity_by_gid(request).await?.into_inner();

        Ok(MultiDimensionalEntity::from_universal_olap_entity(&universal_olap_entity))
    }

    pub async fn locate_universal_olap_entity_by_name(
        &mut self,
        _origin_gid: u64,
        _target_entity_name: &String,
    ) -> Result<MultiDimensionalEntity, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> locate_universal_olap_entity_by_name({}, {})", _origin_gid, _target_entity_name );
        todo!("locate_universal_olap_entity_by_name not implemented yet.");
    }

    pub async fn get_universal_olap_entity_by_gid(
        &mut self,
        gid: u64,
    ) -> Result<MultiDimensionalEntity, Box<dyn std::error::Error>> {
        // println!(">>>>>> Call Meta Server gRPC API >>>>>> get_universal_olap_entity_by_gid({})", gid );
        let request =
            Request::new(GetUniversalOlapEntityByGidRequest { universal_olap_entity_gid: gid });

        let universal_olap_entity =
            self.client.get_universal_olap_entity_by_gid(request).await?.into_inner();

        Ok(MultiDimensionalEntity::from_universal_olap_entity(&universal_olap_entity))
    }

    pub async fn get_all_dimension_roles(
        &mut self,
    ) -> Result<Vec<mdd::DimensionRole>, Box<dyn std::error::Error>> {
        let response = self.client.get_all_dimension_roles(EmptyParameterRequest {}).await?;

        // 将响应数据解析为 DimensionRole 列表
        let dimension_roles: Vec<mdd::DimensionRole> = response
            .into_inner()
            .dimension_roles
            .into_iter()
            .map(|grpc_dr| mdd::DimensionRole {
                gid: grpc_dr.gid,
                // name: grpc_dr.name,
                // cube_gid: grpc_dr.cube_gid,
                dimension_gid: grpc_dr.dimension_gid,
                default_hierarchy_gid: grpc_dr.default_hierarchy_gid,
                measure_flag: grpc_dr.measure_flag == 1,
            })
            .collect();

        Ok(dimension_roles)
    }

    pub async fn get_all_levels(&mut self) -> Result<Vec<mdd::Level>, Box<dyn std::error::Error>> {
        let response = self.client.get_all_levels(EmptyParameterRequest {}).await?;

        let levels: Vec<mdd::Level> = response
            .into_inner()
            .levels
            .into_iter()
            .map(|olap_obj| mdd::Level {
                gid: olap_obj.gid,
                name: olap_obj.name,
                level: olap_obj.level,
                dimension_gid: olap_obj.dimension_gid,
                hierarchy_gid: olap_obj.hierarchy_gid,
                opening_period_gid: olap_obj.opening_period_gid,
                closing_period_gid: olap_obj.closing_period_gid,
            })
            .collect();

        Ok(levels)
    }

    pub async fn get_all_members(
        &mut self,
    ) -> Result<Vec<mdd::Member>, Box<dyn std::error::Error>> {
        let response = self.client.get_all_members(EmptyParameterRequest {}).await?;

        let members: Vec<mdd::Member> = response
            .into_inner()
            .members
            .into_iter()
            .map(|grpc_olap_obj: olapmeta::UniversalOlapEntity| mdd::Member {
                gid: grpc_olap_obj.gid,
                name: grpc_olap_obj.name,
                // dimension_gid: grpc_member.dimension_gid,
                // hierarchy_gid: grpc_member.hierarchy_gid,
                level_gid: grpc_olap_obj.level_gid,
                level: grpc_olap_obj.level,
                parent_gid: grpc_olap_obj.parent_gid,
                measure_index: grpc_olap_obj.measure_index,
                leaf: grpc_olap_obj.leaf,
            })
            .collect();

        Ok(members)
    }

    pub async fn get_all_cubes(&mut self) -> Result<Vec<mdd::Cube>, Box<dyn std::error::Error>> {
        let response = self.client.get_all_cubes(EmptyParameterRequest {}).await?;

        let cubes: Vec<mdd::Cube> = response
            .into_inner()
            .cubes
            .into_iter()
            .map(|grpc_olap_obj: olapmeta::UniversalOlapEntity| mdd::Cube {
                gid: grpc_olap_obj.gid,
                name: grpc_olap_obj.name,
            })
            .collect();

        Ok(cubes)
    }

    pub async fn get_all_formula_members(
        &mut self,
    ) -> Result<Vec<UniversalOlapEntity>, Box<dyn std::error::Error>> {
        let response = self.client.get_all_formula_members(EmptyParameterRequest {}).await?;

        let formula_members: Vec<UniversalOlapEntity> = response
            .into_inner()
            .formula_members
            .into_iter()
            .map(|grpc_olap_obj: UniversalOlapEntity| grpc_olap_obj)
            .collect();

        Ok(formula_members)
    }
}

impl fmt::Debug for GrpcClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<This is a GrpcClient instance.>")
    }
}
