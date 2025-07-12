use crate::mdd::MemberRole;

#[derive(Debug, Clone, PartialEq)]
pub struct TupleVector {
    pub member_roles: Vec<MemberRole>,
}

impl std::fmt::Display for TupleVector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let types_str = self
            .member_roles
            .iter()
            .map(|mr| match mr {
                MemberRole::BaseMember { .. } => "B",
                MemberRole::FormulaMember { .. } => "F",
            })
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "Tuple({})", types_str)
    }
}

impl TupleVector {
    /*
     * self:   [Goods], [Transport], [starting region], [ending region], [starting date], [completion date], [**MeasureDimRole**]
     * other:  [Transport], [completion date], [Goods], [starting region], [ending region]
     * result: [starting date], [**MeasureDimRole**], [Transport], [completion date], [Goods], [starting region], [ending region]
     */
    pub fn merge(&self, other: &Self) -> Self {
        let mut mrs = self.member_roles.clone();
        mrs.retain(|mr| {
            !other
                .member_roles
                .iter()
                .any(|or| or.get_dim_role_gid() == mr.get_dim_role_gid())
        });
        mrs.extend(other.member_roles.clone());

        Self { member_roles: mrs }
    }
}
