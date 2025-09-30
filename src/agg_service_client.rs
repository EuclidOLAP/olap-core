use core::panic;

use std::collections::HashMap;

use agg_service::agg_service_client::AggServiceClient;
use agg_service::{GrpcAggregationRequest, GrpcVectorCoordinate};
use tonic::transport::Channel;

use crate::exmdx::mdd::TupleVector;
use crate::mdd::MemberRole;
use crate::mdd::MultiDimensionalContext;

pub mod agg_service {
    tonic::include_proto!("agg_service");
}

pub struct AggServiceGrpcClient {
    client: AggServiceClient<Channel>,
}

impl AggServiceGrpcClient {
    // Create a new gRPC client instance
    pub async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = AggServiceClient::connect(addr.to_string()).await?;
        Ok(Self { client })
    }

    // Send AggregationRequest and get AggregationResponse
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
    coordinates: Vec<TupleVector>,
    context: &MultiDimensionalContext,
) -> (u64, Vec<f64>, Vec<bool>) {
    if coordinates.is_empty() {
        return (context.cube.gid, vec![], vec![]);
    }

    let coordinates_len = coordinates.len();

    let pass_list: Vec<bool> = context.user_acol.check_access_permission(&coordinates);
    /*
        pass_list中元素数量和coordinates数量相同
        根据pass_list中的值将coordinates分成两部分，创建下面2个变量，在将coordinates分成两部分时还要保留每个元素的初始索引
            let true_indexes: Vec<usize>
            let true_coordinates: Vec<TupleVector>
    */
    let true_elements: Vec<_> = coordinates
        .into_iter()
        .zip(pass_list.into_iter())
        .enumerate()
        .filter(|(_, (_, has_access))| *has_access)
        .collect();

    let true_indexes: Vec<usize> = true_elements.iter().map(|(idx, _)| *idx).collect();
    let true_coordinates: Vec<TupleVector> = true_elements
        .into_iter()
        .map(|(_, (coord, _))| coord)
        .collect();

    if true_indexes.is_empty() {
        // 如果没有有权限的坐标，返回 (context.cube.gid, coordinates 长度的 vec 值都为 0, coordinates 长度的 vec 值都为 true)
        return (
            context.cube.gid,
            vec![0.0; coordinates_len],
            vec![true; coordinates_len],
        );
    }

    let mut grpc_cli = AggServiceGrpcClient::new("http://127.0.0.1:16060")
        .await
        .expect("Failed to create client");

    let gvc_list: Vec<GrpcVectorCoordinate> = transform_coordinates(true_coordinates);

    let result: (u64, Vec<f64>, Vec<bool>) = grpc_cli
        .aggregates(context.cube.gid, gvc_list)
        .await
        .unwrap();

    /*
        根据 true_indexes 创建一个 map，
        key 是 true_indexes 中的索引，value 是 true_indexes 中的具体值
    */
    let mut index_map: HashMap<usize, usize> = HashMap::new();
    for (key, val) in true_indexes.iter().enumerate() {
        index_map.insert(key, *val); // Key 和 Value 都是 idx
    }

    let mut fin_values: Vec<f64> = vec![0.0; coordinates_len];
    let mut fin_null_flags: Vec<bool> = vec![true; coordinates_len];

    let (_, rs_vals, rs_null_fgs) = result;
    for (idx, null_flag) in rs_null_fgs.iter().enumerate() {
        if !(*null_flag) {
            let val = rs_vals[idx];
            let com_index = index_map.get(&idx).unwrap();
            fin_values[*com_index] = val;
            fin_null_flags[*com_index] = false;
        }
    }

    (context.cube.gid, fin_values, fin_null_flags)
}

fn transform_coordinates(coordinates: Vec<TupleVector>) -> Vec<GrpcVectorCoordinate> {
    let mut grpc_coordinates: Vec<GrpcVectorCoordinate> = Vec::new();

    for ocv in coordinates {
        let mut member_roles = ocv.member_roles;
        let mut measure_index: u32 = 0;
        member_roles.retain(|mr| match mr {
            MemberRole::BaseMember { dim_role, member } => {
                if dim_role.measure_flag {
                    measure_index = member.measure_index;
                }
                !dim_role.measure_flag
            }
            MemberRole::FormulaMember { .. } => {
                panic!("FormulaMember is not supported in grpc_client.");
            }
        });
        member_roles.sort_by_key(|mr| mr.get_dim_role_gid());

        let mut gvc = GrpcVectorCoordinate {
            member_gid_arr: vec![],
            measure_index,
        };

        for mr in member_roles {
            if let MemberRole::BaseMember {
                dim_role: _,
                member,
            } = mr
            {
                gvc.member_gid_arr
                    .push(if member.level == 0 { 0 } else { member.gid });
            } else {
                panic!("FormulaMember is not supported in grpc_client.");
            }
        }

        grpc_coordinates.push(gvc);
    }

    grpc_coordinates
}
