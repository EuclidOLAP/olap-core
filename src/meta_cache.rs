use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::mdd::{Cube, Level, Member};
use crate::olapmeta_grpc_client::GrpcClient;
use crate::olapmeta_grpc_client::olapmeta::UniversalOlapEntity;

// 全局线程安全的缓存
static LEVEL_CACHE: Lazy<Mutex<HashMap<u64, Level>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// 全局线程安全的缓存
static MEMBER_CACHE: Lazy<Mutex<HashMap<u64, Member>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// 全局线程安全的缓存
static CUBE_CACHE: Lazy<Mutex<HashMap<u64, Cube>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// 全局线程安全的缓存
static FORMULA_MEMBER_CACHE: Lazy<Mutex<HashMap<u64, UniversalOlapEntity>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 初始化时批量拉取 level 并放入缓存
pub async fn init() {
    let mut grpc_cli = GrpcClient::new("http://192.168.66.51:50051".to_string())
        .await
        .expect("Failed to create client");

    // levels 的类型是 Vec<Level>
    let levels = grpc_cli.get_all_levels().await.unwrap();

    let mut cache = LEVEL_CACHE.lock().unwrap();
    for level in levels {
        // println!("OO>>>>>>>>>>>>>> level: {:#?}", level);
        cache.insert(level.gid, level);
    }

    // members 的类型是 Vec<Member>
    let members = grpc_cli.get_all_members().await.unwrap();

    let mut cache = MEMBER_CACHE.lock().unwrap();
    for member in members {
        // println!("OO>>>>>>>>>>>>>> member: {:#?}", member);
        cache.insert(member.gid, member);
    }

    // cubes 的类型是 Vec<Cube>
    let cubes = grpc_cli.get_all_cubes().await.unwrap();

    let mut cache = CUBE_CACHE.lock().unwrap();
    for cube in cubes {
        // println!("OO>>>>>>>>>>>>>> cube: {:#?}", cube);
        cache.insert(cube.gid, cube);
    }

    let formula_members = grpc_cli.get_all_formula_members().await.unwrap();
    let mut cache = FORMULA_MEMBER_CACHE.lock().unwrap();
    for fm in formula_members {
        // println!(">>>>>>>>>>>>>>>> A Formula Member >>>>>>>>>>>>>>>>>>>>>>>>>>>>\n{:#?}", fm);
        cache.insert(fm.gid, fm);
    }
}

/// 多线程安全地根据 gid 获取 level
pub fn get_level_by_gid(gid: u64) -> Level {
    let cache = LEVEL_CACHE.lock().unwrap();
    match cache.get(&gid) {
        Some(level) => level.clone(),
        None => panic!("Level not found for gid {}", gid),
    }
}

/// 多线程安全地根据 gid 获取 member
pub fn get_member_by_gid(gid: u64) -> Member {
    let cache = MEMBER_CACHE.lock().unwrap();
    match cache.get(&gid) {
        Some(member) => member.clone(),
        None => panic!("Member not found for gid {}", gid),
    }
}

/// 多线程安全地根据 gid 获取 cube
pub fn get_cube_by_gid(gid: u64) -> Cube {
    let cache = CUBE_CACHE.lock().unwrap();
    match cache.get(&gid) {
        Some(cube) => cube.clone(),
        None => panic!("Cube not found for gid {}", gid),
    }
}

pub fn get_hierarchy_level(hierarchy_gid: u64, level_val: u32) -> Level {
    let cache = LEVEL_CACHE.lock().unwrap();
    for level in cache.values() {
        if level.hierarchy_gid == hierarchy_gid && level.level == level_val {
            return level.clone();
        }
    }
    panic!("Level not found for hierarchy_gid = {} and level = {}", hierarchy_gid, level_val);
}

pub fn mdx_formula_members_fragment(cube: &Cube) -> String {
    let cache = FORMULA_MEMBER_CACHE.lock().unwrap();

    let fragments: Vec<String> = cache
        .values()
        .filter(|fm| fm.cube_gid == cube.gid)
        .map(|fm| {
            format!(
                "Member &{}.&{}.&{}[{}] as {}",
                fm.dimension_role_gid, fm.mount_point_gid, fm.gid, fm.name, fm.exp
            )
        })
        .collect();

    fragments.join(",\n")
}
