#[derive(Clone, Debug, PartialEq)]
pub struct AstMdxStatement {}

#[derive(Clone, Debug, PartialEq)]
pub enum AstCustomObj {
    CustomSet,
    CustomMember,
}
