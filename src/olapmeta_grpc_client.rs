// src/olapmeta_grpc_client.rs
use tonic::{transport::Channel, Request};
use std::fmt;
use olapmeta::olap_meta_service_client::OlapMetaServiceClient;
use olapmeta::{CubeGidRequest, CubeNameRequest, CubeMetaResponse};
use olapmeta::GetDimensionRolesByCubeGidRequest;
use olapmeta::GetDefaultDimensionMemberRequest;
use olapmeta::GetDimensionRoleByGidRequest;

use crate::mdd;

pub mod olapmeta {
    // include!("generated/olapmeta.rs"); // 通过 tonic-build 生成的模块
    include!("grpc/olapmeta.rs"); // 通过 tonic-build 生成的模块
}

pub struct GrpcClient {
    client: OlapMetaServiceClient<Channel>,
}

impl GrpcClient {
    // 创建新的客户端实例
    pub async fn new(address: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = OlapMetaServiceClient::connect(address).await?;
        Ok(GrpcClient { client })
    }

    // 通过 GID 获取 Cube
    pub async fn get_cube_by_gid(&mut self, gid: u64) -> Result<CubeMetaResponse, Box<dyn std::error::Error>> {
        let request = Request::new(CubeGidRequest { gid });
        let response = self.client.get_cube_by_gid(request).await?;
        Ok(response.into_inner())
    }

    // 通过 Name 获取 Cube
    pub async fn get_cube_by_name(&mut self, name: String) -> Result<CubeMetaResponse, Box<dyn std::error::Error>> {
        let request = Request::new(CubeNameRequest { name });
        let response = self.client.get_cube_by_name(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_dimension_roles_by_cube_gid(&mut self, cube_gid: u64) -> Result<Vec<mdd::DimensionRole>, Box<dyn std::error::Error>> {
        let response = self.client.get_dimension_roles_by_cube_gid(GetDimensionRolesByCubeGidRequest { gid: cube_gid }).await?;

        // 将响应数据解析为 DimensionRole 列表
        let dimension_roles: Vec<mdd::DimensionRole> = response
            .into_inner()
            .dimension_roles
            .into_iter()
            .map(|grpc_dr| mdd::DimensionRole {
                // gid: grpc_dr.gid,
                // name: grpc_dr.name,
                // cube_gid: grpc_dr.cube_gid,
                dimension_gid: grpc_dr.dimension_gid,
            })
            .collect();

        Ok(dimension_roles)
    }


    pub async fn get_default_dimension_member_by_dimension_gid(&mut self, dimension_gid: u64)
        -> Result<mdd::Member, Box<dyn std::error::Error>> {

        let request = GetDefaultDimensionMemberRequest { dimension_gid };

        let response = self.client.get_default_dimension_member_by_dimension_gid(request).await?;

        let grpc_member = response.into_inner();

        Ok(mdd::Member {
            // gid: grpc_member.gid,
            // name: grpc_member.name,
            // dimension_gid: grpc_member.dimension_gid,
            // hierarchy_gid: grpc_member.hierarchy_gid,
            // level_gid: grpc_member.level_gid,
            // level: grpc_member.level,
            // parent_gid: grpc_member.parent_gid,
        })
    }

    pub async fn get_dimension_role_by_gid(&mut self, dim_role_gid: u64)
        -> Result<mdd::DimensionRole, Box<dyn std::error::Error>> {

            let req = GetDimensionRoleByGidRequest {
                dimension_role_gid: dim_role_gid
            };

            let response = self.client.get_dimension_role_by_gid(req).await?;

            let grpc_dim_role = response.into_inner();

            let dim_role = mdd::DimensionRole {
                dimension_gid: grpc_dim_role.dimension_gid,
            };

            Ok(dim_role)
    }

    pub async fn get_dimension_role_by_name(
        &mut self,
        cube_gid: u64,
        dimension_role_name: &String,
    ) -> Result<mdd::DimensionRole, Box<dyn std::error::Error>> {
        // 构造请求数据
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
            dimension_gid: grpc_dim_role.dimension_gid,
            // 这里可以根据需要添加其他字段
        };
    
        Ok(dim_role)
    }

}

impl fmt::Debug for GrpcClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<This is a GrpcClient instance.>")
    }
}