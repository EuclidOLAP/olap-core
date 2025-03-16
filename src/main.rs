use euclidolap::olap_api_server::{OlapApi, OlapApiServer};
use euclidolap::{GrpcOlapVector, OlapRequest, OlapResponse};
use tonic::{transport::Server, Request, Response, Status};

mod mdd;
use mdd::MemberRole;
use mdd::MultiDimensionalContext;

mod mdx_statements;
mod olapmeta_grpc_client;

mod agg_service_client;
use agg_service_client::basic_aggregates;

mod euclidolap {
    include!("grpc/euclidolap.rs");
}

pub mod mdx_ast;
pub mod mdx_lexer;
pub mod mdx_tokens;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub mdx_grammar);

use crate::mdx_grammar::SelectionMDXParser;

use crate::mdx_lexer::Lexer as MdxLexer;

use mdd::OlapVectorCoordinate;

#[derive(Debug, Default)]
pub struct EuclidOLAPService {}

#[tonic::async_trait]
impl OlapApi for EuclidOLAPService {
    async fn execute_operation(
        &self,
        request: Request<OlapRequest>, // 从客户端接收的请求
    ) -> Result<Response<OlapResponse>, Status> {
        // println!("Received request: {:?}", request);

        // 从请求中解析操作类型和语句
        let olap_request = request.into_inner();
        let operation_type = olap_request.operation_type;
        let statement = olap_request.statement;

        // println!(
        //     "Operation Type: {}, Statement: >>>>>>{}<<<<<<",
        //     operation_type, statement
        // );

        let mut olap_resp = OlapResponse { vectors: vec![] };

        let (_cube_gid, measures_values, null_flags) = handle_stat(operation_type, statement).await;

        for (val, null_flag) in measures_values.into_iter().zip(null_flags.into_iter()) {
            let vector = GrpcOlapVector { null_flag, val };
            olap_resp.vectors.push(vector);
        }

        Ok(Response::new(olap_resp))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 定义服务端监听地址
    // let addr = "127.0.0.1:50052".parse().unwrap();
    let addr = "0.0.0.0:50052".parse().unwrap();

    // 创建 Greeter 服务实例
    let olap_api_server = EuclidOLAPService::default();

    println!(">>> EuclidOLAP Server is listening on {} <<<", addr);

    // 启动 gRPC 服务端
    Server::builder()
        .add_service(OlapApiServer::new(olap_api_server)) // 添加 Greeter 服务
        .serve(addr) // 启动服务端
        .await?;

    Ok(())
}

async fn handle_stat(optype: String, statement: String) -> (u64, Vec<f64>, Vec<bool>) {
    match optype.as_str() {
        "MDX" => {
            println!(">>>>>>>> MDX Statement >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
            println!("{}", statement);
            println!(">>>>>>>> <<<<<<<<<<<<< >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
            let ast_selstat = SelectionMDXParser::new()
                .parse(MdxLexer::new(statement.as_str()))
                .unwrap();

            exe_md_query(ast_selstat).await
        }
        _ => {
            panic!(
                "In fn `handle_stat()`: Unexpected operation type: {}",
                optype
            );
        }
    }
}

async fn exe_md_query(ast_selstat: mdx_ast::AstSelectionStatement) -> (u64, Vec<f64>, Vec<bool>) {
    let mut context = ast_selstat.gen_md_context().await;
    let axes = ast_selstat.build_axes(&mut context).await;
    let coordinates: Vec<OlapVectorCoordinate> =
        mdd::Axis::axis_vec_cartesian_product(&axes, &context);

    // Base OlapVectorCoordinates and Formula OlapVectorCoordinates
    // 分别存储索引和坐标数据
    let mut base_indices: Vec<usize> = Vec::new();
    let mut frml_indices: Vec<usize> = Vec::new();
    let mut base_cords: Vec<OlapVectorCoordinate> = Vec::new();
    let mut frml_cords: Vec<OlapVectorCoordinate> = Vec::new();

    'outside: for (idx, cord) in coordinates.into_iter().enumerate() {
        for mr in &cord.member_roles {
            if let MemberRole::FormulaMember{dim_role_gid: _, exp: _} = mr {
                frml_indices.push(idx);
                frml_cords.push(cord);
                continue 'outside;
            }
        }
        base_indices.push(idx);
        base_cords.push(cord);
    }

    // basic_aggregates(coordinates, &context).await
    let (cube_gid, base_vals, base_null_flags) = basic_aggregates(base_cords, &context).await;
    let (_, frml_vals, frml_null_flags) = calculate_formula_vectors(frml_cords, &context);


    // 初始化最终的合并结果，确保索引对应数据不乱序
    let mut merged_vals = vec![0.0; base_indices.len() + frml_indices.len()];
    let mut merged_null_flags = vec![false; base_indices.len() + frml_indices.len()];

    // 填充基本数据
    for (idx, value) in base_indices.iter().zip(base_vals.iter()) {
        merged_vals[*idx] = *value;
    }
    for (idx, flag) in base_indices.iter().zip(base_null_flags.iter()) {
        merged_null_flags[*idx] = *flag;
    }

    // 填充公式数据
    for (idx, value) in frml_indices.iter().zip(frml_vals.iter()) {
        merged_vals[*idx] = *value;
    }
    for (idx, flag) in frml_indices.iter().zip(frml_null_flags.iter()) {
        merged_null_flags[*idx] = *flag;
    }

    (cube_gid, merged_vals, merged_null_flags)

}

fn calculate_formula_vectors(
    coordinates: Vec<OlapVectorCoordinate>,
    context: &MultiDimensionalContext,
) -> (u64, Vec<f64>, Vec<bool>) {

    (context.cube.gid, vec![660880.5; coordinates.len()], vec![false; coordinates.len()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mdx_statements::*;

    // #[test]
    fn _test_handle_stat() {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_stat(String::from("MDX"), _mdx_demo()));
    }

    // #[test]
    fn _test_handle_stat_2() {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_stat(String::from("MDX"), _mdx_demo_2_axposstr()));
    }

    // #[test]
    fn _test_grpc_locate_gid_1() {
        async fn test_grpc_locate_gid_1_async() {
            use mdd::MultiDimensionalEntity;
            use olapmeta_grpc_client::GrpcClient;

            let params: [(u64, u64); 9] = [
                (600000000000023, 300000000004617),
                (600000000000022, 300000000004173),
                (600000000000015, 300000000004175),
                (600000000000015, 300000000004072),
                (600000000000015, 300000000004164),
                (600000000000008, 300000000004531),
                (600000000000011, 300000000000003),
                (600000000000023, 300000000004612),
                (600000000000024, 300000000004612),
            ];

            let mut grpc_cli = GrpcClient::new("http://192.168.66.51:50051".to_string())
                .await
                .expect("Failed to create client");

            for (origin_gid, target_entity_gid) in params {
                let olap_entity = grpc_cli
                    .locate_universal_olap_entity_by_gid(origin_gid, target_entity_gid)
                    .await
                    .unwrap();

                match olap_entity {
                    MultiDimensionalEntity::MemberWrap(member) => {
                        println!(">>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>> Member: \n{:#?}", member);
                    }
                    _ => {
                        panic!("Unexpected olap_entity type: {:#?}", olap_entity);
                    }
                }
            }
        }

        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(test_grpc_locate_gid_1_async());
    }

    #[test]
    fn test_mdx_3() {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_stat(String::from("MDX"), _mdx_3()));
    }
}
