#[derive(Clone, Debug, PartialEq)]
pub struct AstMdxStatement {
    cus_objs: Option<Vec<AstCustomObj>>,
    axes: Vec<AstAxis>,
    segs_cube: AstSegsObj,
    where_slicing: Option<AstTuple>,
}

impl AstMdxStatement {
    pub fn new(
        cus_objs: Option<Vec<AstCustomObj>>,
        axes: Vec<AstAxis>,
        segs_cube: AstSegsObj,
        where_slicing: Option<AstTuple>,
    ) -> Self {
        Self { cus_objs, axes, segs_cube, where_slicing }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstCustomObj {
    CustomMember(AstSegsObj, AstExp),
    CustomSetBySegsObj(AstSegsObj, AstSegsObj),
    CustomSetByTuples(AstSegsObj, AstTuples),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSeg {
    Gid(u64),
    GidStr(u64, String),
    Str(String),
    MemberFunc(AstMemberRoleFunc),
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

#[derive(Clone, Debug, PartialEq)]
pub enum AstAxis {
    CurlyBraceSet(AstTuples, u64),
    SegsObjSet(AstSegsObj, u64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstExp {
    pub terms: Vec<(char, AstTerm)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstTerm {
    pub factories: Vec<(char, AstFac)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstFac {
    Double(f64),
    Str(String),
    SegsObj(AstSegsObj),
    Tuple(AstTuple),
    Exp(AstExp),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberRoleFunc {
    Parent(AstMemRoleFnParent),
    CurrentMember(AstMemRoleFnCurrentMember),
    OpeningPeriod(AstMemRoleFnOpeningPeriod),
    ClosingPeriod(AstMemRoleFnClosingPeriod),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnParent {
    Simple,
    MemberRoleSegs(AstSegsObj),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnCurrentMember {
    Simple,
    DimRoleSegs(AstSegsObj),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnOpeningPeriod {
    Simple,
    LvRoleSegs(AstSegsObj),
    LvRoleSegsMemRoleSegs(AstSegsObj, AstSegsObj),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemRoleFnClosingPeriod {
    Simple,
    LvRoleSegs(AstSegsObj),
    LvRoleSegsMemRoleSegs(AstSegsObj, AstSegsObj),
}
