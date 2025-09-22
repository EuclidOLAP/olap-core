pub mod permis_obj;

use crate::olapmeta_grpc_client::GrpcClient;

use crate::cfg::get_cfg;

#[derive(Debug, Clone)]
pub struct UserAccessesCollection {}

impl UserAccessesCollection {
    pub async fn new(user_name: String) -> Self {
        let mut meta_grpc_cli = GrpcClient::new(get_cfg().meta_grpc_url)
            .await
            .expect("Failed to create client");

        let _acces = meta_grpc_cli
            .load_user_olap_model_accesses(user_name)
            .await
            .unwrap();

        // println!("\n\n\n< 2 > acces: {:#?}\n\n\n", acces);
        Self {}
    }
}
