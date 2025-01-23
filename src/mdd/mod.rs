#[derive(Debug)]
pub struct MultiDimensionalContext {
    pub cube: Cube,
    pub ref_tuple: Tuple, // defautl slice tuple
}

#[derive(Debug)]
pub struct Tuple {}

// #[derive(Debug)]
// pub struct MemberRole {}

#[derive(Debug)]
pub struct DimensionRole {
    // pub gid: u64,
    // pub name: String,
    // pub cube_gid: u64,
    pub dimension_gid: u64,
}

// #[derive(Debug)]
// pub struct Dimension {}

#[derive(Debug)]
pub struct Member {
    // pub gid: u64,
    // pub name: String,
    // pub dimension_gid: u64,
    // pub hierarchy_gid: u64,
    // pub level_gid: u64,
    // pub level: u64,
    // pub parent_gid: u64,
}

#[derive(Debug)]
pub struct Cube {
    pub gid: u64,
    pub name: String,
}

pub struct Axis {
    pub pos_num: u32,
}
