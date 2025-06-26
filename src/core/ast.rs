#[derive(Clone, Debug, PartialEq)]
pub struct AstMdxStatement {}

#[derive(Clone, Debug, PartialEq)]
pub enum AstCustomObj {
    CustomSet,
    CustomMember,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSeg {
    Gid(u64),
    GidStr(u64, String),
    Str(String),
    MemberFunc(()),
    LevelFunc(()),
    SetFunc(()),
    ExpFunc(()),
}