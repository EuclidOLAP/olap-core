use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::cfg::get_cfg;

use crate::mdd::{Cube, Level, Member};
use crate::olapmeta_grpc_client::olapmeta::UniversalOlapEntity;
use crate::olapmeta_grpc_client::GrpcClient;

// 全局线程安全的缓存
static LEVEL_CACHE: Lazy<Mutex<HashMap<u64, Level>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// 全局线程安全的缓存
static MEMBER_CACHE: Lazy<Mutex<HashMap<u64, Member>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// 全局线程安全的缓存
static CUBE_CACHE: Lazy<Mutex<HashMap<u64, Cube>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// 全局线程安全的缓存
static FORMULA_MEMBER_CACHE: Lazy<Mutex<HashMap<u64, UniversalOlapEntity>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 初始化时批量拉取 level 并放入缓存
pub async fn init() {
    let config = get_cfg();
    println!("< 2 > config.meta_grpc_url: {:#?}", config.meta_grpc_url);

    let mut grpc_cli = GrpcClient::new(config.meta_grpc_url)
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
    panic!(
        "Level not found for hierarchy_gid = {} and level = {}",
        hierarchy_gid, level_val
    );
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

/// 获取指定 level_gid 上的所有成员，按 gid 排序返回。
///
/// 从 MEMBER_CACHE 中筛选出 level_gid 相同的所有成员，
/// 按 gid 排序后返回。这样可以保证同一层级的成员有确定的顺序。
pub fn get_members_at_level(level_gid: u64) -> Vec<Member> {
    let cache = MEMBER_CACHE.lock().unwrap();
    let mut members: Vec<Member> = cache
        .values()
        .filter(|m| m.level_gid == level_gid)
        .cloned()
        .collect();
    
    // 按 gid 排序，确保顺序稳定
    members.sort_by_key(|m| m.gid);
    members
}

/// 返回给定 `member_gid` 在指定 `level_gid` 上的祖先 Member。
///
/// 实现策略：使用内存缓存 `MEMBER_CACHE`，通过 member.parent_gid 向上遍历，
/// 直到找到 level_gid 相等的 member 并返回它。如果遍历到 root（parent_gid == 0）仍未找到，
/// 则 panic（调用方应保证请求的 level 是该 member 的上级层次之一）。
pub fn get_member_ancestor_on_level(member_gid: u64, level_gid: u64) -> Member {
    // Start from the provided member and walk up using cached members.
    let mut cur_gid = member_gid;

    loop {
        let cache = MEMBER_CACHE.lock().unwrap();
        let member = match cache.get(&cur_gid) {
            Some(m) => m.clone(),
            None => panic!("Member not found for gid {} when searching ancestor", cur_gid),
        };

        if member.level_gid == level_gid {
            return member;
        }

        if member.parent_gid == 0 {
            panic!("Ancestor at level_gid {} not found for member {}", level_gid, member_gid);
        }

        cur_gid = member.parent_gid;
    }
}

/*
    实现一个函数，此函数功能是平行移动祖先member，并获得相同位置的member
    参数说明：
        ancestor_member：祖先member
        member：原始member
        offset：偏移量，正数表示向前移动，负数表示向后移动
    函数逻辑：
        ancestor_member是member的某个层级的祖先，
        member相对于ancestor_member的位置可以描述为一个整形列表，
        假设结构如下，
            ancestor_member
                ├── child1
                │    ├── grandchild1
                │    └── grandchild2
                └── child2
                     ├── grandchild3
                     └── grandchild4
        那么grandchild3相对于ancestor_member的位置可以表示为[1, 0]，
        则[1, 0]就是member相对于ancestor_member的位置描述列表，
        在获得位置描述后，根据offset找到ancestor_member平移后的另一个祖先member，
        然后在另一个祖先member下，根据位置描述列表找到对应的member并返回。
 */

/// 在祖先节点上平移并取得对应位置的成员。
///
/// 参数：
/// - `grpc_cli`: 已初始化的 `GrpcClient`（可变引用），用于按顺序获取某个 member 的子成员列表（以保留目录顺序）。
/// - `ancestor_gid`: 被视为祖先的 member 的 gid（必须是 `member_gid` 的某一上层）。
/// - `member_gid`: 原始的后代 member gid。
/// - `offset`: 偏移量，正数表示向前（索引减小）移动，负数表示向后（索引增大）移动。
///
/// 实现：
/// 1. 检查 `ancestor_gid` 是否确实出现在 `member_gid` 的 `full_path` 中（否则 panic）。
/// 2. 从 `member_gid` 向上遍历到 `ancestor_gid`，在每一层通过 `grpc_cli.get_child_members_by_gid(parent_gid)`
///    获取子成员列表以确定当前成员在父节点下的顺序索引，记录下这些索引（从祖先的子层开始到目标成员的路径）。
/// 3. 在祖先的父节点下获取祖先的兄弟列表，按 `offset` 平移得到新的祖先。
/// 4. 从新的祖先按步骤 2 中记录的索引逐层向下查找对应的成员并返回。
///
/// 注意：该函数在未找到预期成员或索引越界时会 panic（与项目中其他缓存/辅助函数风格保持一致）。
pub async fn shift_ancestor_and_find_member(
    grpc_cli: &mut GrpcClient,
    ancestor_gid: u64,
    member_gid: u64,
    offset: i32,
) -> Member {
    // 验证 ancestor 是 member 的祖先（使用 full_path）
    let member = get_member_by_gid(member_gid);
    if !member.full_path.contains(&ancestor_gid) {
        panic!(
            "Ancestor gid {} is not an ancestor of member {}",
            ancestor_gid, member_gid
        );
    }

    // 从 member 向上到 ancestor，记录每一级在其父节点下的索引（从下向上）
    let mut indices_rev: Vec<usize> = Vec::new();
    let mut cur_gid = member_gid;

    while cur_gid != ancestor_gid {
        let cur = get_member_by_gid(cur_gid);
        let parent_gid = cur.parent_gid;

        // 获取 parent 的子成员（有序）
        let children = grpc_cli.get_child_members_by_gid(parent_gid).await.unwrap();
        // 找到 cur_gid 在 children 中的索引
        let mut found_idx: Option<usize> = None;
        for (i, ch) in children.iter().enumerate() {
            if ch.gid == cur_gid {
                found_idx = Some(i);
                break;
            }
        }
        let idx = match found_idx {
            Some(i) => i,
            None => panic!("Failed to locate member {} under parent {}", cur_gid, parent_gid),
        };
        indices_rev.push(idx);

        cur_gid = parent_gid;
        if cur_gid == 0 {
            panic!("Reached root while searching for ancestor {}", ancestor_gid);
        }
    }

    // 现在 indices_rev 列表包含从 member 向上至 ancestor 的索引（child index），需要反转为从 ancestor 向下
    indices_rev.reverse();

    // 在祖先的父节点下找到祖先的兄弟并应用 offset
    let ancestor = get_member_by_gid(ancestor_gid);
    let ancestor_parent_gid = ancestor.parent_gid;
    let siblings = grpc_cli.get_child_members_by_gid(ancestor_parent_gid).await.unwrap();

    let mut ancestor_index: Option<usize> = None;
    for (i, s) in siblings.iter().enumerate() {
        if s.gid == ancestor_gid {
            ancestor_index = Some(i);
            break;
        }
    }
    let ancestor_index = ancestor_index.expect("Ancestor not found among its parent's children");

    // offset: 按调用约定，正数表示向前移动（索引减小），负数表示向后移动（索引增大）——与 MDX ParallelPeriod 的常见语义保持一致
    let target_idx_signed = ancestor_index as isize - offset as isize;
    if target_idx_signed < 0 || target_idx_signed as usize >= siblings.len() {
        panic!(
            "Shifted ancestor index out of range: base_index={} offset={} result_index={}",
            ancestor_index, offset, target_idx_signed
        );
    }
    let target_ancestor = siblings[target_idx_signed as usize].clone();

    // 从 target_ancestor 向下按 indices_rev 路径查找最终成员
    let mut cur = target_ancestor;
    for idx in indices_rev.iter() {
        let children = grpc_cli.get_child_members_by_gid(cur.gid).await.unwrap();
        if *idx >= children.len() {
            panic!("Child index {} out of range for parent {}", idx, cur.gid);
        }
        cur = children[*idx].clone();
    }

    cur
}