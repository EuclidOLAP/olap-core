use crate::mdd;
use crate::olapmeta_grpc_client::GrpcClient;

#[derive(Clone, Debug, PartialEq)]
pub enum ExtMDXStatement {
    Querying { basic_cube: AstCube },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstCube {}

#[derive(Clone, Debug, PartialEq)]
pub struct AstSeg {
    pub gid: Option<u64>,
    pub seg_str: Option<String>,
}

impl AstSeg {
    pub async fn materialize(&self, context: &mut mdd::MultiDimensionalContext) /*-> MultiDimensionalEntity*/ {
        println!("AstSeg::materialize() >---------------------------------");

        // 由于是在多维查询上下文中，所以一般应该返回带有角色信息的实体
        // 首先判断是否有 gid，如果有，则通过 gid 查询，如果没有，则通过 seg_str 查询
        match (self.gid, &self.seg_str) {
            (Some(gid), _) => {
                context.find_entity_by_gid(gid).await;
                println!("/////////////////////////////////////////// context.find_entity_by_gid( {} );", gid);
            },
            (None, Some(seg_str)) => {
                context.find_entity_by_str(seg_str).await;
                println!("/////////////////////////////////////////// context.find_entity_by_str( {} );", seg_str);
            },
            (None, None) => {
                panic!("Both gid and seg_str are None, cannot query!");
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSegments {
    Segs(Vec<AstSeg>),
}

impl AstSegments {
    pub async fn materialize(&self, context: &mut mdd::MultiDimensionalContext) /* -> MultiDimensionalEntity */ {
        println!("AstSegments::materialize() >>>>>>>>>>");
        match self {
            AstSegments::Segs(segs) => {
                let ast_seg = segs.iter().next().unwrap();
                ast_seg.materialize(context).await;
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstTuple {
    SegsList(Vec<AstSegments>),
}

impl AstTuple {
    pub async fn materialize(&self, context: &mut mdd::MultiDimensionalContext) {
        println!("AstTuple::materialize() >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
        match self {
            AstTuple::SegsList(segs_list) => {
                let ast_segments = segs_list.iter().next().unwrap();
                ast_segments.materialize(context).await;
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSet {
    Tuples(Vec<AstTuple>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstAxis {
    Def { ast_set: AstSet, pos: u64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstSelectionStatement {
    pub axes: Vec<AstAxis>,
    pub cube: Vec<AstSeg>,
    pub basic_slice: Option<AstTuple>,
}

impl AstSelectionStatement {
    pub async fn gen_md_context(&self) -> mdd::MultiDimensionalContext {
        // 获取真正的 Cube 实例
        let cube_pro = &self.cube;
        let ast_seg_opt = cube_pro.get(0);

        // 初始化默认 Cube
        let cube;

        // 创建 gRPC 客户端
        let mut grpc_cli = GrpcClient::new("http://192.168.66.51:50051".to_string())
            .await
            .expect("Failed to create client");

        // 如果没有 ast_seg，直接 panic
        let ast_seg = match ast_seg_opt {
            Some(ast_seg) => ast_seg,
            None => panic!("In method AstSelectionStatement::gen_md_context(): cube is empty!"),
        };

        // 通过 gid 或 seg_str 查询 Cube
        let gid_opt = ast_seg.gid;

        if let Some(gid) = gid_opt {
            // println!("CCD >>> gid: {}", gid);
            cube = self.fetch_cube_by_gid(&mut grpc_cli, gid).await;
        } else {
            let seg_str_opt = &ast_seg.seg_str;
            let seg_str = seg_str_opt.as_ref().unwrap_or_else(|| {
                panic!("In method AstSelectionStatement::gen_md_context(): cube seg_str is empty!")
            });
            cube = self.fetch_cube_by_name(&mut grpc_cli, seg_str).await;
        }

        let mut cube_def_tuple = mdd::Tuple {
            member_roles: Vec::new(),
        };

        let dimension_roles = grpc_cli
            .get_dimension_roles_by_cube_gid(cube.gid)
            .await
            .unwrap();
        for dim_role in dimension_roles {
            let dim_def_member = grpc_cli
                .get_default_dimension_member_by_dimension_gid(dim_role.dimension_gid)
                .await
                .unwrap();

            cube_def_tuple.member_roles.push(mdd::MemberRole {
                dim_role,
                member: dim_def_member,
            });
        }

        mdd::MultiDimensionalContext {
            cube,
            ref_tuple: cube_def_tuple,
            grpc_client: grpc_cli,
        }
    }

    async fn fetch_cube_by_gid(&self, grpc_cli: &mut GrpcClient, gid: u64) -> mdd::Cube {
        match grpc_cli.get_cube_by_gid(gid).await {
            Ok(response) => response
                .cube_meta
                .map(|meta| mdd::Cube {
                    gid: meta.gid,
                    name: meta.name,
                })
                .unwrap_or_else(|| {
                    println!("Error fetching Cube by GID: CubeMeta is None");
                    mdd::Cube {
                        gid: 0,
                        name: String::from(">>> No cube found <<<"),
                    }
                }),
            Err(e) => {
                println!("Error fetching Cube by GID: {}", e);
                mdd::Cube {
                    gid: 0,
                    name: String::from(">>> No cube found <<<"),
                }
            }
        }
    }

    async fn fetch_cube_by_name(&self, grpc_cli: &mut GrpcClient, name: &str) -> mdd::Cube {
        match grpc_cli.get_cube_by_name(name.to_string()).await {
            Ok(response) => {
                println!("Received Cube by Name: {:?}", response);
                response
                    .cube_meta
                    .map(|meta| mdd::Cube {
                        gid: meta.gid,
                        name: meta.name,
                    })
                    .unwrap_or_else(|| {
                        println!("Error fetching Cube by Name: CubeMeta is None");
                        mdd::Cube {
                            gid: 0,
                            name: String::from(">>> No cube found <<<"),
                        }
                    })
            }
            Err(e) => {
                println!("Error fetching Cube by Name: {}", e);
                mdd::Cube {
                    gid: 0,
                    name: String::from(">>> No cube found <<<"),
                }
            }
        }
    }

    pub async fn build_axes(&self, context: &mut mdd::MultiDimensionalContext) -> Vec<mdd::Axis> {
        println!("AstSelectionStatement::build_axes() >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");

        // MDX语句中是否包含where
        if let Some(slice) = &self.basic_slice {
            slice.materialize(context).await;
        } else {

        }


        // println!(
        //     "build_axes .. . ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~"
        // );

        let axes_count = self.axes.len();
        // println!(
        //     ">>> axes_count >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>: {}",
        //     axes_count
        // );

        let mut axes: Vec<mdd::Axis> = Vec::with_capacity(axes_count);

        for i in 0..axes_count {
            let axis = mdd::Axis { pos_num: i as u32 };
            axes.push(axis);
        }

        axes
    }
}

// #[derive(Clone, Debug, PartialEq)]
// pub enum Statement {
//     Variable {
//         name: String,
//         value: Box<Expression>,
//     },
//     Print {
//         value: Box<Expression>,
//     },
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum Expression {
//     Integer(i64),
//     Variable(String),
//     BinaryOperation {
//         lhs: Box<Expression>,
//         operator: Operator,
//         rhs: Box<Expression>,
//     },
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum Operator {
//     Add,
//     Sub,
//     Mul,
//     Div,
//     #[cfg(feature = "bit")]
//     Shl,
// }
