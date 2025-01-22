// src/olapmeta_grpc_client.rs
use tonic::{transport::Channel, Request};
use olapmeta::olap_meta_service_client::OlapMetaServiceClient;
use olapmeta::{CubeGidRequest, CubeNameRequest, CubeMetaResponse};
use olapmeta::GetDimensionRolesByCubeGidRequest;

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
                gid: grpc_dr.gid,
                name: grpc_dr.name,
                cube_gid: grpc_dr.cube_gid,
                dimension_gid: grpc_dr.dimension_gid,
            })
            .collect();

        Ok(dimension_roles)
    }
}
