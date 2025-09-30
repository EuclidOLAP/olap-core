pub mod permis_obj;

use crate::permission::permis_obj::UserOlapModelAccess;

use crate::cfg::get_cfg;
use crate::mdd::GidType;
use crate::olapmeta_grpc_client::GrpcClient;

use crate::meta_cache;

use crate::exmdx::mdd::TupleVector;

#[derive(Debug, Clone)]
pub struct UserAccessesCollection {
    user_accoll: Vec<UserOlapModelAccess>,
}

impl UserAccessesCollection {
    pub async fn new(user_name: String) -> Self {
        let mut meta_grpc_cli = GrpcClient::new(get_cfg().meta_grpc_url)
            .await
            .expect("Failed to create client");

        let user_accoll = meta_grpc_cli
            .load_user_olap_model_accesses(user_name)
            .await
            .unwrap();

        Self { user_accoll }
    }

    pub fn check_access_permission(&self, coordinates: &Vec<TupleVector>) -> Vec<bool> {
        let result = coordinates
            .iter()
            .map(|tuple| self.check_permission_for_tuple(tuple))
            .collect();

        result
    }

    fn check_permission_for_tuple(&self, tuple: &TupleVector) -> bool {
        // println!(">>>>>> fn check_permission_for_tuple(&self, tuple: &TupleVector) -> bool");
        for member_role in tuple.member_roles.iter() {
            // println!(">>>>>> >>>>>> member_role: {:?}", member_role);
            if let crate::mdd::MemberRole::FormulaMember { .. } = member_role {
                // 公式成员暂时不做权限校验，目前什么也不用做
                continue;
            }

            let crate::mdd::MemberRole::BaseMember { dim_role, member } = member_role else {
                panic!("[20250928152507] impossible!");
            };

            if dim_role.measure_flag {
                println!("// todo - bug-num-20250930110604");
                continue;
            }

            let mut acc_permis_defined: Option<(u32, bool)> = None;

            for oma in &self.user_accoll {
                // println!(">>>>>> >>>>>> >>>>>> oma: {:?}", oma);
                // 如果定义在cube，直接 continue
                if GidType::entity_type(oma.olap_entity_gid) == GidType::Cube {
                    continue;
                }

                if oma.dimension_role_gid != dim_role.gid {
                    // 维度角色不匹配，继续下一个
                    continue;
                }

                let oma_olap_entity_gid: u64 = oma.olap_entity_gid;
                let member_full_path: &Vec<u64> = &member.full_path;
                let does_have_access = oma.has_access;

                if GidType::entity_type(oma.olap_entity_gid) == GidType::Member {
                    if meta_cache::get_member_by_gid(oma.olap_entity_gid).level == 0 {
                        // it is a root member
                        if let None = acc_permis_defined {
                            acc_permis_defined = Some((0_u32, does_have_access));
                            // println!(">>>>>> >>>>>> >>>>>> >>>>>> [root & none] acc_permis_defined = {:?}", acc_permis_defined);
                        }
                        continue;
                    }
                }

                let posi_option = member_full_path
                    .iter()
                    .position(|gid| *gid == oma_olap_entity_gid);

                if let Some(pos) = posi_option {

                    let pos_ignore_root = pos as u32 + 1;

                    match acc_permis_defined {
                        None => {
                            // 第一次匹配，直接记录
                            acc_permis_defined = Some((pos_ignore_root, does_have_access));
                            // println!(">>>>>> >>>>>> >>>>>> >>>>>> [none -> some] acc_permis_defined = {:?}", acc_permis_defined);
                        }
                        Some((existing_pos, _)) => {
                            // 已经有一个匹配了，取路径上更远的（索引更大）
                            if pos_ignore_root > existing_pos {
                                acc_permis_defined = Some((pos_ignore_root, does_have_access));
                                // println!(">>>>>> >>>>>> >>>>>> >>>>>> [change] acc_permis_defined = {:?}", acc_permis_defined);
                            }
                        }
                    }
                }
            }

            // 执行到这里，只有明确定义了可访问权限，才继续下一个 member_role，否则返回 false
            if let Some((_, true)) = acc_permis_defined {
                continue;
            } else {
                // println!(">>>>>> return false;");
                return false;
            }
        }
        // println!(">>>>>> return true;");
        // 能执行到这里，说明没有任何权限限制，返回 true
        true
    }
}
