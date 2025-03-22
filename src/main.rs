use euclidolap::olap_api_server::{OlapApi, OlapApiServer};
use euclidolap::{GrpcOlapVector, OlapRequest, OlapResponse};
use tonic::{transport::Server, Request, Response, Status};

mod mdd;

pub mod calcul;

mod olapmeta_grpc_client;

mod agg_service_client;

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

    calcul::calculate(coordinates, &mut context).await
}
