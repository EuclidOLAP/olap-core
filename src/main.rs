mod agg_service_client;
// mod core;
mod exmdx;
mod cache;
mod mdd;
mod meta_cache;
mod olapmeta_grpc_client;

mod euclidolap {
    tonic::include_proto!("euclidolap");
}

use crate::exmdx::mdd::TupleVector;

pub mod calcul;
pub mod cfg;
pub mod mdx_ast;
pub mod mdx_lexer;
pub mod mdx_tokens;

lalrpop_mod!(pub mdx_grammar);

use euclidolap::olap_api_server::{OlapApi, OlapApiServer};
use euclidolap::{GrpcOlapVector, OlapRequest, OlapResponse};
use mdd::CellValue;
use tonic::{transport::Server, Request, Response, Status};

use lalrpop_util::lalrpop_mod;

// use crate::mdx_grammar::EuclidMdxStatementParser;
use crate::mdx_grammar::SelectionMDXParser;

use crate::mdx_lexer::Lexer as MdxLexer;

// use mdd::OlapVectorCoordinate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    cache::meta::reload().await;

    meta_cache::init().await;

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

#[derive(Debug, Default)]
pub struct EuclidOLAPService {}

#[tonic::async_trait]
impl OlapApi for EuclidOLAPService {
    async fn execute_operation(
        &self,
        request: Request<OlapRequest>, // 从客户端接收的请求
    ) -> Result<Response<OlapResponse>, Status> {
        // 从请求中解析操作类型和语句
        let olap_request = request.into_inner();
        let operation_type = olap_request.operation_type;
        let statement = olap_request.statement;

        let (_cube_gid, cell_vals) = handle_stat(operation_type, statement).await;

        let grpc_olap_vectors: Vec<GrpcOlapVector> = cell_vals
            .iter()
            .map(|cell| match cell {
                CellValue::Double(val) => {
                    GrpcOlapVector { null_flag: false, val: *val, str: format!("{}", *val) }
                }
                CellValue::Str(str) => {
                    GrpcOlapVector { null_flag: false, val: 0.0, str: String::from(str) }
                }
                CellValue::Null => {
                    GrpcOlapVector { null_flag: true, val: 0.0, str: String::from("") }
                }
                CellValue::Invalid => {
                    GrpcOlapVector { null_flag: false, val: 0.0, str: String::from("Invalid") }
                }
            })
            .collect();

        let olap_resp = OlapResponse { vectors: grpc_olap_vectors };

        Ok(Response::new(olap_resp))
    }
}

async fn handle_stat(optype: String, statement: String) -> (u64, Vec<CellValue>) {
    match optype.as_str() {
        "MDX" => {
            // println!(">>>>>>>> MDX Statement >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
            // println!("{}", statement);
            // println!(">>>>>>>> <<<<<<<<<<<<< >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

            // let ast_selstat =
            //     EuclidMdxStatementParser::new().parse(MdxLexer::new(&statement)).unwrap();
            // println!("[Cyrex] EuclidMdxStatementParser >>>>>> {:?}", ast_selstat);

            let ast_selstat = SelectionMDXParser::new().parse(MdxLexer::new(&statement)).unwrap();

            exe_md_query(ast_selstat).await
        }
        _ => {
            panic!("In fn `handle_stat()`: Unexpected operation type: {}", optype);
        }
    }
}

async fn exe_md_query(ast_selstat: mdx_ast::AstSelectionStatement) -> (u64, Vec<CellValue>) {
    let mut context = ast_selstat.gen_md_context().await;
    let axes = ast_selstat.build_axes(&mut context).await;
    let coordinates: Vec<TupleVector> =
        mdd::Axis::axis_vec_cartesian_product(&axes, &context);

    (context.cube.gid, calcul::calculate(coordinates, &mut context).await)
}
