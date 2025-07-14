use futures::future::BoxFuture;

use crate::mdx_ast::AstExpression;
use crate::mdx_ast::AstSeg;
use crate::mdx_ast::Materializable;

use crate::exmdx::mdd::TupleVector;
use crate::mdd::GidType;
use crate::mdd::MemberRole;
use crate::mdd::MultiDimensionalContext;
use crate::mdd::MultiDimensionalEntity;
use crate::mdd::MultiDimensionalEntityLocator;
use crate::mdd::{Axis, Cube, Set};

use core::panic;
use std::collections::HashMap;

use crate::cfg::get_cfg;
use crate::olapmeta_grpc_client::GrpcClient;

#[derive(Clone, Debug, PartialEq)]
pub struct AstSegsObj {
    pub segs: Vec<AstSeg>,
}

impl AstSegsObj {
    pub fn new(seg: AstSeg) -> Self {
        Self { segs: vec![seg] }
    }

    pub fn append(&mut self, seg: AstSeg) {
        self.segs.push(seg);
    }

    fn get_pos_gid(&self, pos: usize) -> Option<u64> {
        self.segs.get(pos)?.get_gid()
    }

    pub fn get_last_gid(&self) -> Option<u64> {
        self.get_pos_gid(self.segs.len() - 1)
    }

    pub fn get_first_gid(&self) -> Option<u64> {
        self.get_pos_gid(0)
    }
}

impl Materializable for AstSegsObj {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            let mut is_formula_member = false;

            let last_opt = self.get_last_gid();
            if let Some(last_gid) = last_opt {
                is_formula_member = GidType::entity_type(last_gid) == GidType::FormulaMember;
            }

            if is_formula_member {
                let dim_role_gid = self.get_first_gid().unwrap();

                let cus_obj: AstCustomObject = context
                    .formulas_map
                    .get(&last_opt.unwrap())
                    .unwrap()
                    .clone();
                if let AstCustomObject::FormulaMember(_, exp) = cus_obj {
                    // let AstCustomObject::FormulaMember(_, exp) =
                    //     context.formulas_map.get(&last_opt.unwrap()).unwrap().clone();
                    return MultiDimensionalEntity::FormulaMemberWrap { dim_role_gid, exp };
                } else {
                    todo!("[NVB676] MemberRoleWrap is not implemented yet.")
                }
            }

            let ast_seg = self.segs.iter().next().unwrap();
            let head_entity: MultiDimensionalEntity =
                ast_seg.materialize(slice_tuple, context).await;

            if self.segs.len() == 1 {
                return head_entity;
            }

            match head_entity {
                MultiDimensionalEntity::DimensionRoleWrap(dim_role) => {
                    let tail_segs = AstSegsObj {
                        segs: (self.segs[1..]).to_vec(),
                    };

                    dim_role
                        .locate_entity(&tail_segs, slice_tuple, context)
                        .await
                }
                MultiDimensionalEntity::MemberRoleWrap(member_role) => {
                    if self.segs.len() == 1 {
                        return MultiDimensionalEntity::MemberRoleWrap(member_role);
                    }
                    todo!("[NVB676] MemberRoleWrap is not implemented yet.")
                }
                MultiDimensionalEntity::LevelRole(lv_role) => {
                    if self.segs.len() == 1 {
                        return MultiDimensionalEntity::LevelRole(lv_role);
                    }
                    todo!("[NVB666DC] MemberRoleWrap is not implemented yet.")
                }
                MultiDimensionalEntity::Cube(cube) => {
                    if self.segs.len() == 1 {
                        // return MultiDimensionalEntity::Cube(cube);
                        MultiDimensionalEntity::Cube(cube)
                    } else {
                        let tail_segs = AstSegsObj {
                            segs: (self.segs[1..]).to_vec(),
                        };
                        cube.locate_entity(&tail_segs, slice_tuple, context).await
                    }
                }
                MultiDimensionalEntity::SetWrap(set) => {
                    let tail_segs = AstSegsObj {
                        segs: (self.segs[1..]).to_vec(),
                    };
                    set.locate_entity(&tail_segs, slice_tuple, context).await
                }
                _ => {
                    panic!("In method AstSegsObj::materialize(): head_entity is not a DimensionRoleWrap!");
                }
            }
        })
    }
}

pub struct AstMdxStatement {
    pub custom_objs: Vec<AstCustomObject>,
    pub axes: Vec<AstAxis>,
    pub cube_segs: AstSegsObj,
    pub slicing: Option<AstTuple>,
}

impl AstMdxStatement {
    pub fn new(
        custom_objs: Vec<AstCustomObject>,
        slicing_querying: (Vec<AstAxis>, AstSegsObj, Option<AstTuple>),
    ) -> Self {
        let (axes, cube_segs, slicing) = slicing_querying;
        Self {
            custom_objs,
            axes,
            cube_segs,
            slicing,
        }
    }
}

impl AstMdxStatement {
    pub async fn gen_md_context(&self) -> MultiDimensionalContext {
        // 获取真正的 Cube 实例
        let cube_pro = &self.cube_segs.segs;
        let ast_seg_opt = cube_pro.get(0);

        // 初始化默认 Cube
        let cube;

        let config = get_cfg();
        println!("< 1 > config.meta_grpc_url: {:#?}", config.meta_grpc_url);

        // 创建 gRPC 客户端
        let mut grpc_cli = GrpcClient::new(config.meta_grpc_url)
            .await
            .expect("Failed to create client");

        // 如果没有 ast_seg，直接 panic
        let ast_seg = match ast_seg_opt {
            Some(ast_seg) => ast_seg,
            None => panic!("In method AstSelectionStatement::gen_md_context(): cube is empty!"),
        };

        match ast_seg {
            AstSeg::Gid(gid) => {
                cube = self.fetch_cube_by_gid(&mut grpc_cli, *gid).await;
            }
            AstSeg::Str(seg_str) => {
                cube = self.fetch_cube_by_name(&mut grpc_cli, &seg_str).await;
            }
            AstSeg::GidStr(gid, _) => {
                cube = self.fetch_cube_by_gid(&mut grpc_cli, *gid).await;
            }
            _ => panic!("The entity is not a Gid or a Str variant. 2"),
        }

        let mut cube_def_tuple = TupleVector {
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

            cube_def_tuple.member_roles.push(MemberRole::BaseMember {
                dim_role,
                member: dim_def_member,
            });
        }

        let mut formulas_map: HashMap<u64, AstCustomObject> = HashMap::new();
        for frml_obj in &self.custom_objs {
            match frml_obj {
                AstCustomObject::FormulaMember(segments, _exp) => {
                    let frml_member_gid = segments.get_last_gid().unwrap();
                    formulas_map.insert(frml_member_gid, frml_obj.clone());
                    // let frml_member_gid = segments.get_last_gid().unwrap();
                    // formulas_map.insert(frml_member_gid, frml_obj.clone());
                }
                AstCustomObject::CustomSet(_cus_set_segs, _ast_set) => {
                    panic!("CustomSet is not supported yet.")
                }
            }
        }

        let mut context = MultiDimensionalContext {
            cube,
            // cube_def_tuple,
            // where_tuple: None,
            query_slice_tuple: TupleVector {
                member_roles: vec![],
            },
            grpc_client: grpc_cli,
            formulas_map,
        };

        let mut where_tuple: Option<TupleVector> = None;
        if let Some(mdx_where) = &self.slicing {
            where_tuple = match mdx_where.materialize(&cube_def_tuple, &mut context).await {
                MultiDimensionalEntity::TupleWrap(tuple) => Some(tuple),
                _ => panic!("The entity is not a TupleWrap variant."),
            }
        };

        // context.where_tuple = where_tuple;

        let mut query_slice_tuple = cube_def_tuple.clone();
        if let Some(where_tuple) = &where_tuple {
            query_slice_tuple = query_slice_tuple.merge(where_tuple);
        }
        context.query_slice_tuple = query_slice_tuple;

        context
    }

    async fn fetch_cube_by_gid(&self, grpc_cli: &mut GrpcClient, gid: u64) -> Cube {
        match grpc_cli.get_cube_by_gid(gid).await {
            Ok(response) => response
                .cube_meta
                .map(|meta| Cube {
                    gid: meta.gid,
                    name: meta.name,
                })
                .unwrap_or_else(|| {
                    println!("Error fetching Cube by GID: CubeMeta is None");
                    Cube {
                        gid: 0,
                        name: String::from(">>> No cube found <<<"),
                    }
                }),
            Err(e) => {
                println!("Error fetching Cube by GID: {}", e);
                Cube {
                    gid: 0,
                    name: String::from(">>> No cube found <<<"),
                }
            }
        }
    }

    async fn fetch_cube_by_name(&self, grpc_cli: &mut GrpcClient, name: &str) -> Cube {
        match grpc_cli.get_cube_by_name(name.to_string()).await {
            Ok(response) => {
                println!("Received Cube by Name: {:?}", response);
                response
                    .cube_meta
                    .map(|meta| Cube {
                        gid: meta.gid,
                        name: meta.name,
                    })
                    .unwrap_or_else(|| {
                        println!("Error fetching Cube by Name: CubeMeta is None");
                        Cube {
                            gid: 0,
                            name: String::from(">>> No cube found <<<"),
                        }
                    })
            }
            Err(e) => {
                println!("Error fetching Cube by Name: {}", e);
                Cube {
                    gid: 0,
                    name: String::from(">>> No cube found <<<"),
                }
            }
        }
    }

    pub async fn build_axes(&self, context: &mut MultiDimensionalContext) -> Vec<Axis> {
        let mut slice_tuple = context.query_slice_tuple.clone();

        let axes_count = self.axes.len();

        for _ in 0..axes_count {
            for ast_axis in self.axes.iter() {
                let fiducial_tuple = ast_axis
                    .generate_fiducial_tuple(&slice_tuple, context)
                    .await;
                slice_tuple = slice_tuple.merge(&fiducial_tuple);
            }
        }

        let mut axes: Vec<Axis> = Vec::with_capacity(axes_count);

        for ast_axis in self.axes.iter() {
            let axis: Axis = ast_axis.translate_to_axis(&slice_tuple, context).await;
            axes.push(axis);
        }

        axes
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSet {
    Tuples(Vec<AstTuple>),
    SegsObj(AstSegsObj),
}

impl AstSet {
    async fn generate_fiducial_tuple(
        &self,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> TupleVector {
        let result;
        match self {
            AstSet::Tuples(tuples) => {
                result = match tuples
                    .iter()
                    .next()
                    .unwrap()
                    .materialize(slice_tuple, context)
                    .await
                {
                    MultiDimensionalEntity::TupleWrap(tuple) => tuple.clone(),
                    _ => panic!("The entity is not a TupleWrap variant."),
                };
            }
            AstSet::SegsObj(_segs_obj) => {
                panic!("AstSet::SegsObj is not implemented yet.")
            }
        }
        result
    }
}

impl Materializable for AstSet {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            let mut tuple_vec: Vec<TupleVector> = Vec::new();

            match self {
                AstSet::Tuples(tuples) => {
                    for ast_tuple in tuples.iter() {
                        let tuple_entity = ast_tuple.materialize(slice_tuple, context).await;
                        match tuple_entity {
                            MultiDimensionalEntity::TupleWrap(tuple) => {
                                tuple_vec.push(tuple);
                            }
                            _ => {
                                panic!("The entity is not a TupleWrap variant.");
                            }
                        }
                    }
                }
                _ => {
                    panic!("AstSet::SegsObj is not implemented yet.")
                }
            }

            MultiDimensionalEntity::SetWrap(Set { tuples: tuple_vec })
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstTuple {
    SegsObjects(Vec<AstSegsObj>),
    SegsObj(AstSegsObj),
}

impl Materializable for AstTuple {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a TupleVector,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            match self {
                AstTuple::SegsObjects(segs_list) => {
                    let mut member_roles: Vec<MemberRole> = Vec::new();
                    for segs in segs_list.iter() {
                        let member_role_entity = segs.materialize(slice_tuple, context).await;
                        match member_role_entity {
                            MultiDimensionalEntity::MemberRoleWrap(member_role) => {
                                member_roles.push(member_role);
                            }
                            MultiDimensionalEntity::FormulaMemberWrap { dim_role_gid, exp } => {
                                member_roles.push(MemberRole::FormulaMember { dim_role_gid, exp });
                            }
                            _ => {
                                panic!("The entity is not a MemberRoleWrap variant.");
                            }
                        }
                    }
                    MultiDimensionalEntity::TupleWrap(TupleVector { member_roles })
                }
                AstTuple::SegsObj(_segs) => {
                    panic!("AstTuple::SegsObj is not implemented yet.")
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstCustomObject {
    FormulaMember(AstSegsObj, AstExpression),
    CustomSet(AstSegsObj, AstSet),
}

// #[derive(Clone, Debug, PartialEq)]
pub enum AstAxis {
    SetDefinition { ast_set: AstSet, pos: u64 },
}

impl AstAxis {
    async fn generate_fiducial_tuple(
        &self,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> TupleVector {
        match self {
            AstAxis::SetDefinition { ast_set, pos: _ } => {
                ast_set.generate_fiducial_tuple(slice_tuple, context).await
            }
        }
    }

    async fn translate_to_axis(
        &self,
        slice_tuple: &TupleVector,
        context: &mut MultiDimensionalContext,
    ) -> Axis {
        let axis: Axis;

        match self {
            AstAxis::SetDefinition { ast_set, pos } => {
                let olap_entity = ast_set.materialize(slice_tuple, context).await;
                match olap_entity {
                    MultiDimensionalEntity::SetWrap(set) => {
                        axis = Axis {
                            set,
                            pos_num: *pos as u32,
                        };
                    }
                    _ => {
                        panic!("The entity is not a SetWrap variant.");
                    }
                }
            }
        }
        axis
    }
}
