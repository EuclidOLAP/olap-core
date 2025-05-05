use crate::olapmeta_grpc_client::GrpcClient;

pub async fn init() {
    println!("Initializing meta_cache module......... ...... ...");

    let mut grpc_cli = GrpcClient::new("http://192.168.66.51:50051".to_string())
        .await
        .expect("Failed to create client");

    println!("grpc_cli: {:#?}", grpc_cli);
    println!("meta_cache module initialized successfully &&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&...............");

    let dimension_roles = grpc_cli.get_all_dimension_roles().await.unwrap();

    println!(">>>>>>>");
    println!(">>>>>>>>>>>>>>");
    println!(">>>>>>>>>>>>>>>>>>>>>");
    println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>> !?");

    for role in dimension_roles {
        println!("DimensionRole: {:?}", role);
    }

    println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>> Okay");
    println!(">>>>>>>>>>>>>>>>>>>>>");
    println!(">>>>>>>>>>>>>>");
    println!(">>>>>>>");
}
