use crate::cache::MEMBER_CACHE;
use crate::mdd::Member;
use crate::olapmeta_grpc_client::GrpcClient;

pub async fn reload() {
    let mut meta_grpc_cli = GrpcClient::get_cli().await;
    let members = meta_grpc_cli.get_all_members().await.unwrap();
    let mut cache = MEMBER_CACHE.write().unwrap();
    cache.clear();
    for member in members {
        cache.insert(member.gid, member); // 将每个 member 按 gid 存入缓存
    }
    println!("Successfully reloaded members into the cache.");
}

pub fn _get_gid_member<'cache_period>(_gid: u64) -> Option<&'cache_period Member> {
    // let cache = MEMBER_CACHE.read().unwrap();
    // cache.get(&gid)
    todo!()
}
