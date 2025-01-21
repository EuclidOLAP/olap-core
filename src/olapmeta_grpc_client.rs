// src/olapmeta_grpc_client.rs
use tonic::{transport::Channel, Request};
use olapmeta::olap_meta_service_client::OlapMetaServiceClient;
use olapmeta::{CubeGidRequest, CubeNameRequest, CubeMetaResponse};

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
}
