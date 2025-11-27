// message GrpcUserOlapModelAccess {
//     uint64 id                 = 1,
//     string user_name          = 2,
//     string permission_scope   = 3,
//     uint64 dimension_role_gid = 4,
//     uint64 olap_entity_gid    = 5,
//     bool has_access           = 6,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct UserOlapModelAccess {
    pub id: u64,
    pub user_name: String,
    pub permission_scope: String,
    pub dimension_role_gid: u64,
    pub olap_entity_gid: u64,
    pub has_access: bool,
}
