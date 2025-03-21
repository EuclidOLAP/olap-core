use core::panic;

use agg_service::agg_service_client::AggServiceClient;
use agg_service::{GrpcAggregationRequest, GrpcVectorCoordinate};
use tonic::transport::Channel;

use crate::mdd::MultiDimensionalContext;
use crate::mdd::OlapVectorCoordinate;
use crate::mdd::MemberRole;

pub mod agg_service {
    include!("grpc/agg_service.rs"); // 通过 tonic-build 生成的模块
}

pub struct AggServiceGrpcClient {
    client: AggServiceClient<Channel>,
}

impl AggServiceGrpcClient {
    /// 创建新的 gRPC 客户端实例
    pub async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = AggServiceClient::connect(addr.to_string()).await?;
        Ok(Self { client })
    }

    /// 发送 AggregationRequest 并获取 AggregationResponse
    pub async fn aggregates(
        &mut self,
        cube_gid: u64,
        coordinates: Vec<GrpcVectorCoordinate>,
    ) -> Result<(u64, Vec<f64>, Vec<bool>), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GrpcAggregationRequest {
            cube_gid,
            grpc_vector_coordinates: coordinates,
        });

        let response = self.client.aggregates(request).await?.into_inner();

        Ok((response.cube_gid, response.values, response.null_flags))
    }
}

pub async fn basic_aggregates(
    coordinates: Vec<OlapVectorCoordinate>,
    context: &MultiDimensionalContext,
) -> (u64, Vec<f64>, Vec<bool>) {

    if coordinates.is_empty() {
        return (context.cube.gid, vec![], vec![]);
    }

    let mut grpc_cli = AggServiceGrpcClient::new("http://127.0.0.1:16060")
        .await
        .expect("Failed to create client");

    let gvc_list: Vec<GrpcVectorCoordinate> = transform_coordinates(coordinates);

    let cube_gid = context.cube.gid;

    let result = grpc_cli.aggregates(cube_gid, gvc_list).await.unwrap();

    result
}

fn transform_coordinates(coordinates: Vec<OlapVectorCoordinate>) -> Vec<GrpcVectorCoordinate> {
    let mut grpc_coordinates: Vec<GrpcVectorCoordinate> = Vec::new();

    for ocv in coordinates {
        let mut member_roles = ocv.member_roles;
        let mut measure_index: u32 = 0;
        member_roles.retain(|mr| {
            match mr {
                MemberRole::BaseMember{dim_role,member} => {
                    if dim_role.measure_flag {
                        measure_index = member.measure_index;
                    }
                    !dim_role.measure_flag
                },
                MemberRole::FormulaMember{..} => {
                    panic!("FormulaMember is not supported in grpc_client.");
                }
            }
        });
        member_roles.sort_by_key(|mr| mr.get_dim_role_gid () ) ;

        let mut gvc = GrpcVectorCoordinate {
            member_gid_arr: vec![],
            measure_index,
        };

        for mr in member_roles {
            if let MemberRole::BaseMember{dim_role: _, member} = mr {
                gvc.member_gid_arr.push(if member.level == 0 { 0 } else { member.gid });
            } else {
                panic!("FormulaMember is not supported in grpc_client.");
            }
        }

        grpc_coordinates.push(gvc);
    }

    grpc_coordinates
}
