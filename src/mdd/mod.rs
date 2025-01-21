#[derive(Debug)]
pub struct MultiDimensionalContext {
    pub cube: Cube,
}

#[derive(Debug)]
pub struct Cube {
    pub gid: u64,
    pub name: String,
}

pub struct Axis {
    pub pos_num: u32,
}