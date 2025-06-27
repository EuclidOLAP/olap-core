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

#[derive(Clone, Debug, PartialEq)]
pub struct AstSegsObj {
    segs: Vec<AstSeg>,
}

impl AstSegsObj {
    pub fn new(seg: AstSeg) -> Self {
        Self { segs: vec![seg] }
    }

    pub fn append(&mut self, seg: AstSeg) {
        self.segs.push(seg)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstTuples {
    ast_tuples: Vec<AstTuple>,
}

impl AstTuples {
    pub fn new(astup: AstTuple) -> Self {
        Self { ast_tuples: vec![astup] }
    }

    pub fn append(&mut self, astup: AstTuple) {
        self.ast_tuples.push(astup)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstTuple {
    RoundBracketTuple(Vec<AstSegsObj>),
    SegsObj(AstSegsObj),
}
