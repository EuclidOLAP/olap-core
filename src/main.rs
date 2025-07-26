mod agg_service_client;
// mod core;
mod cache;
mod exmdx;
mod mdd;
mod meta_cache;
mod olapmeta_grpc_client;

mod euclidolap {
    tonic::include_proto!("euclidolap");
}

use crate::exmdx::ast::AstMdxStatement;
use crate::exmdx::mdd::TupleVector;

pub mod calcul;
pub mod cfg;
// pub mod mdx_ast;
pub mod mdx_lexer;
pub mod mdx_tokens;

lalrpop_mod!(pub mdx_grammar);

use euclidolap::olap_api_server::{OlapApi, OlapApiServer};
use euclidolap::{GrpcOlapVector, OlapRequest, OlapResponse};
use mdd::CellValue;
use tonic::{transport::Server, Request, Response, Status};

use lalrpop_util::lalrpop_mod;

use crate::mdx_grammar::MdxStatementParser;

use crate::mdx_lexer::Lexer as MdxLexer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cache::meta::reload().await;

    meta_cache::init().await;

    // let addr = "127.0.0.1:50052".parse().unwrap();
    let addr = "0.0.0.0:50052".parse().unwrap();

    let olap_api_server = EuclidOLAPService::default();

    println!(">>> EuclidOLAP Server is listening on {} <<<", addr);

    // 启动 gRPC 服务端
    Server::builder()
        .add_service(OlapApiServer::new(olap_api_server)) // 添加 Greeter 服务
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct EuclidOLAPService {}

#[tonic::async_trait]
impl OlapApi for EuclidOLAPService {
    async fn execute_operation(
        &self,
        request: Request<OlapRequest>,
    ) -> Result<Response<OlapResponse>, Status> {
        let olap_request = request.into_inner();
        let operation_type = olap_request.operation_type;
        let statement = olap_request.statement;

        let (_cube_gid, cell_vals) = handle_stat(operation_type, statement).await;

        let grpc_olap_vectors: Vec<GrpcOlapVector> = cell_vals
            .iter()
            .map(|cell| match cell {
                CellValue::Double(val) => GrpcOlapVector {
                    null_flag: false,
                    val: *val,
                    str: format!("{}", *val),
                },
                CellValue::Str(str) => GrpcOlapVector {
                    null_flag: false,
                    val: 0.0,
                    str: String::from(str),
                },
                CellValue::Null => GrpcOlapVector {
                    null_flag: true,
                    val: 0.0,
                    str: String::from(""),
                },
                CellValue::Invalid => GrpcOlapVector {
                    null_flag: false,
                    val: 0.0,
                    str: String::from("Invalid"),
                },
            })
            .collect();

        let olap_resp = OlapResponse {
            vectors: grpc_olap_vectors,
        };

        Ok(Response::new(olap_resp))
    }
}

async fn handle_stat(optype: String, statement: String) -> (u64, Vec<CellValue>) {
    match optype.as_str() {
        "MDX" => {
            let ast_selstat = MdxStatementParser::new()
                .parse(MdxLexer::new(&statement))
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

async fn exe_md_query(ast_selstat: AstMdxStatement) -> (u64, Vec<CellValue>) {
    let mut context = ast_selstat.gen_md_context().await;
    let axes = ast_selstat.build_axes(&mut context).await;
    let coordinates: Vec<TupleVector> = mdd::Axis::axis_vec_cartesian_product(&axes, &context);

    (
        context.cube.gid,
        calcul::calculate(coordinates, &mut context).await,
    )
}
