use euclidolap::olap_api_server::{OlapApi, OlapApiServer};
use euclidolap::{OlapRequest, OlapResponse, Row};
use tonic::{transport::Server, Request, Response, Status};

// mod mdx_parser;

mod euclidolap {
    include!("grpc/euclidolap.rs");
}

pub mod mdx_ast;
pub mod mdx_lexer;
pub mod mdx_tokens;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub mdx_grammar);

use crate::mdx_grammar::MdxStatementParser;
// use crate::mdx_grammar::BlocksChainParser;
use crate::mdx_grammar::AxesParser;
use crate::mdx_grammar::AxisParser;
use crate::mdx_grammar::SegParser;
use crate::mdx_grammar::SegmentsParser;
use crate::mdx_grammar::SegmentsWrapParser;
use crate::mdx_grammar::SelectionMDXParser;
use crate::mdx_grammar::SetWrapParser;
use crate::mdx_grammar::TupleWrapParser;

use crate::mdx_lexer::Lexer as MdxLexer;

#[derive(Debug, Default)]
pub struct EuclidOLAPService {}

#[tonic::async_trait]
impl OlapApi for EuclidOLAPService {
    async fn execute_operation(
        &self,
        request: Request<OlapRequest>, // 从客户端接收的请求
    ) -> Result<Response<OlapResponse>, Status> {
        println!("Received request: {:?}", request);

        // 从请求中解析操作类型和语句
        let olap_request = request.into_inner();
        let operation_type = olap_request.operation_type;
        let statement = olap_request.statement;

        // println!(
        //     "Operation Type: {}, Statement: >>>>>>{}<<<<<<",
        //     operation_type, statement
        // );

        handle_stat(operation_type, statement);

        // 伪造一个响应，返回结果
        let response = OlapResponse {
            rows: vec![
                Row {
                    columns: vec![
                        "[ cell: R 0 C 0 ]".to_string(),
                        "[ cell: R 0 C 1 ]".to_string(),
                        "[ cell: R 0 C 2 ]".to_string(),
                    ],
                },
                Row {
                    columns: vec![
                        "[ cell: R 1 C 0 ]".to_string(),
                        "[ cell: R 1 C 1 ]".to_string(),
                        "[ cell: R 1 C 2 ]".to_string(),
                    ],
                },
                Row {
                    columns: vec![
                        "AAAsssDDD".to_string(),
                        "111222333".to_string(),
                        "@@@###$$$".to_string(),
                    ],
                },
            ],
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    test_parsing_mdx_01();
    test_parsing_mdx_02();
    test_parsing_mdx_03();
    test_parsing_mdx_04();
    test_parsing_mdx_05();
    test_parsing_mdx_06();
    test_parsing_mdx_07();
    test_parsing_mdx_08();
    test_parsing_mdx_09();
    // ????????????????????????????????????????????????????????????????????

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

fn handle_stat(optype: String, statement: String) -> () {
    match optype.as_str() {
        "MDX" => {
            println!("\n\n\n>>> @@@ Operation Type: {}", optype);
            println!(">>> @@@ Statement: {}", statement);
        },
        _ => {
            panic!("In fn `handle_stat()`: Unexpected operation type: {}", optype);
        }
    }
    ()
}

// #[test]
fn test_parsing_mdx_01() {
    let source_code = "Select --{}\n on from where";

    let lexer = MdxLexer::new(source_code);
    let parser = MdxStatementParser::new();
    let ast = parser.parse(lexer).unwrap();

    println!("MDX---------------------------------------{:?}", ast);
}

// #[test]
fn test_parsing_mdx_02() {
    let ast_node = SegParser::new()
        .parse(MdxLexer::new("&0000000000000000000000000300000000321"))
        .unwrap();
    println!("MDX TEST 02 >>>>>>>>>>>>>>>>>>>>> {:?}", ast_node);
    let ast_node = SegParser::new()
        .parse(MdxLexer::new(
            "&0000000000000000000000000000000000000000000000000000000000000000000300000000345[LA]",
        ))
        .unwrap();
    println!("MDX TEST 02 >>>>>>>>>>>>>>>>>>>>> {:?}", ast_node);
    let ast_node = SegParser::new()
        .parse(MdxLexer::new("[[iPhone 16]] pro max]"))
        .unwrap();
    println!("MDX TEST 02 >>>>>>>>>>>>>>>>>>>>> {:?}", ast_node);
}

// Segments
// #[test]
fn test_parsing_mdx_03() {
    println!(">>> mdx_test_03_segments >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let ast_node = SegmentsParser::new()
        .parse(MdxLexer::new("&100000123.&100000123.&100000123[>[[]]]]<]"))
        .unwrap();
    println!(">>> mdx_test_03_segments {:?}", ast_node);
    let ast_node = SegmentsParser::new()
        .parse(MdxLexer::new("[丰田].&100000123[兰德酷路泽].&100000123"))
        .unwrap();
    println!(">>> mdx_test_03_segments {:?}", ast_node);
}

// SegmentsWrapParser
// #[test]
fn test_parsing_mdx_04() {
    println!(">>> 04 SegmentsWrapParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let ast_node = SegmentsWrapParser::new()
        .parse(MdxLexer::new("&100000123.&100000123.&100000123[>[[]]]]<]"))
        .unwrap();
    println!(">>> 04 SegmentsWrapParser {:?}", ast_node);
}

// TupleWrapParser
// #[test]
fn test_parsing_mdx_05() {
    println!("\n\n");

    println!(">>> 05 TupleWrapParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let ast_node = TupleWrapParser::new()
        .parse(MdxLexer::new(
            "( [丰田].&100000123[兰德酷路泽].&100000123, [丰田].&100000123[兰德酷路泽], [丰田] )",
        ))
        .unwrap();
    println!("{:?}", ast_node);

    println!("\n\n");
}

// SetWrapParser
// #[test]
fn test_parsing_mdx_06() {
    println!("\n\n");

    println!(">>> 06 SetWrapParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    let mdx_fragment = "{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004692[企业客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
}";
    let ast_node = SetWrapParser::new()
        .parse(MdxLexer::new(mdx_fragment))
        .unwrap();
    println!("{:?}", ast_node);

    println!("\n\n");
}

// AxisParser
// #[test]
fn test_parsing_mdx_07() {
    println!("\n\n");
    println!(">>> 07 AxisParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

    let mdx_fragment = "{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
} on 090";
    let ast_node = AxisParser::new()
        .parse(MdxLexer::new(mdx_fragment))
        .unwrap();
    println!("{:?}", ast_node);

    println!("\n\n");
}

// AxesParser
// #[test]
fn test_parsing_mdx_08() {
    println!("\n\n");
    println!(">>> 08 AxesParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

    let mdx_fragment = "{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004692[企业客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
}
on 1,
{
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] )
}
on 0000";
    let ast_node = AxesParser::new()
        .parse(MdxLexer::new(mdx_fragment))
        .unwrap();

    println!("{:?}", ast_node);
    println!("\n\n");
}

// SelectionMDXParser
// #[test]
fn test_parsing_mdx_09() {
    println!("\n\n");
    println!(">>> 09 SelectionMDXParser >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

    let mdx_fragment = "select
{
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004718[Online Store], &600000000000006[22].&300000000004692[企业客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004685[VIP客户] ),
( &600000000000007[333].&300000000004719[Retail Store], &600000000000006[22].&300000000004692[企业客户] )
}
on 1,
{
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004570[中国], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004670[Credit Card], &600000000000001[rr].&300000000001641[2024-05] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001515[2024] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001548[2024-02] ),
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] )
}
on 0
from &500000000000001[多维立方体]
where
( &600000000000004[gggg].&300000000004571[上海], &600000000000005[11].&300000000004672[PayPal], &600000000000001[rr].&300000000001641[2024-05] )";
    let ast_node = SelectionMDXParser::new()
        .parse(MdxLexer::new(mdx_fragment))
        .unwrap();

    println!("{:?}", ast_node);
    println!("\n\n");
}
