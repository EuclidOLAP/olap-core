use std::collections::HashMap;
use std::vec;

use crate::mdd;
use crate::mdd::MultiDimensionalEntityLocator;
use crate::mdd::MultiDimensionalContext;
use crate::mdd::OlapVectorCoordinate;
use crate::mdd::{MultiDimensionalEntity, Tuple, GidType, MemberRole};
use crate::olapmeta_grpc_client::GrpcClient;

use crate::calcul::calculate;

// Temporarily allow async fn in trait to suppress the compiler warning.
// This trait is currently used only within this project, so auto trait bounds (e.g., Send) are not a concern.
// TODO: If this trait is made public or used in a multi-threaded context, 
// consider refactoring async fn into a regular fn returning `impl Future + Send` to ensure thread safety.
#[allow(async_fn_in_trait)]
pub trait Materializable {
    async fn materialize(
        &self,
        slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> MultiDimensionalEntity;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExtMDXStatement {
    Querying { basic_cube: AstCube },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstCube {}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSeg {
    Gid(u64),
    Str(String),
    GidStr(u64, String),
    MemberFunction(AstMemberFunction),
}

impl AstSeg {
    pub fn get_gid(&self) -> Option<u64> {
        match self {
            AstSeg::Gid(gid) => Some(*gid),
            AstSeg::GidStr(gid, _) => Some(*gid),
            _ => None,
        }
    }
}

impl Materializable for AstSeg {
    async fn materialize(
        &self,
        _slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        // 由于是在多维查询上下文中，所以一般应该返回带有角色信息的实体
        // 首先判断是否有 gid，如果有，则通过 gid 查询，如果没有，则通过 seg_str 查询
        match self {
            AstSeg::Gid(gid) => context.find_entity_by_gid(*gid).await,
            AstSeg::Str(seg_str) => context.find_entity_by_str(seg_str).await,
            AstSeg::GidStr(gid, _) => context.find_entity_by_gid(*gid).await,
            _ => panic!("The entity is not a Gid or a Str variant. 1"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSegments {
    Segs(Vec<AstSeg>),
}

impl AstSegments {

    fn get_pos_gid(&self, pos: usize) -> Option<u64> {
        match self {
            AstSegments::Segs(segs) => {
                let pos_seg = segs.get(pos)?;
                pos_seg.get_gid()
            }
        }
    }

    pub fn get_last_gid(&self) -> Option<u64> {
        match self {
            AstSegments::Segs(segs) => {
                self.get_pos_gid(segs.len() - 1)
            }
        }
    }
    
    pub fn get_first_gid(&self) -> Option<u64> {
        self.get_pos_gid(0)
    }
}

impl Materializable for AstSegments {
    async fn materialize(
        &self,
        slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match self {
            AstSegments::Segs(segs) => {

                let mut is_formula_member = false;

                let last_opt = self.get_last_gid();
                if let Some(last_gid) = last_opt {
                    if GidType::entity_type(last_gid) == GidType::FormulaMember {
                        is_formula_member = true;
                    }
                }

                if is_formula_member {
                    let dim_role_gid = self.get_first_gid().unwrap();
                    let AstFormulaObject::CustomFormulaMember(_, exp) = context.formulas_map.get(&last_opt.unwrap()).unwrap().clone();
                    return MultiDimensionalEntity::FormulaMemberWrap{ dim_role_gid, exp };
                }

                let mut segs_iter = segs.iter();
                let ast_seg = segs_iter.next().unwrap();
                let head_entity: MultiDimensionalEntity =
                    ast_seg.materialize(slice_tuple, context).await;

                match head_entity {
                    MultiDimensionalEntity::DimensionRoleWrap(dim_role) => {
                        let tail_segs = AstSegments::Segs((&segs[1..]).to_vec());
                        dim_role .locate_entity(&tail_segs, slice_tuple, context) .await
                    }
                    _ => {
                        panic!("In method AstSegments::materialize(): head_entity is not a DimensionRoleWrap!");
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstTuple {
    SegsList(Vec<AstSegments>),
}

impl Materializable for AstTuple {
    async fn materialize(
        &self,
        slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match self {
            AstTuple::SegsList(segs_list) => {
                let mut member_roles: Vec<mdd::MemberRole> = Vec::new();
                for segs in segs_list.iter() {
                    let member_role_entity = segs.materialize(slice_tuple, context).await;
                    match member_role_entity {
                        MultiDimensionalEntity::MemberRoleWrap(member_role) => {
                            member_roles.push(member_role);
                        }
                        MultiDimensionalEntity::FormulaMemberWrap{dim_role_gid, exp} => {
                            member_roles.push(MemberRole::FormulaMember { dim_role_gid, exp });
                        }
                        _ => {
                            panic!("The entity is not a MemberRoleWrap variant.");
                        }
                    }
                }
                MultiDimensionalEntity::TupleWrap(mdd::Tuple { member_roles })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSet {
    Tuples(Vec<AstTuple>),
}

impl AstSet {
    async fn generate_fiducial_tuple(
        &self,
        slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> mdd::Tuple {
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
        }
        result
    }
}

impl Materializable for AstSet {
    async fn materialize(
        &self,
        slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        let mut tuple_vec: Vec<Tuple> = Vec::new();

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
        }

        MultiDimensionalEntity::SetWrap(mdd::Set { tuples: tuple_vec })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstAxis {
    SetDefinition { ast_set: AstSet, pos: u64 },
}

impl AstAxis {
    async fn generate_fiducial_tuple(
        &self,
        slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> mdd::Tuple {
        match self {
            AstAxis::SetDefinition { ast_set, pos: _ } => {
                ast_set.generate_fiducial_tuple(slice_tuple, context).await
            }
        }
    }

    async fn translate_to_axis(
        &self,
        slice_tuple: &Tuple,
        context: &mut mdd::MultiDimensionalContext,
    ) -> mdd::Axis {
        let axis: mdd::Axis;

        match self {
            AstAxis::SetDefinition { ast_set, pos } => {
                let olap_entity = ast_set.materialize(slice_tuple, context).await;
                match olap_entity {
                    MultiDimensionalEntity::SetWrap(set) => {
                        axis = mdd::Axis {
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

#[derive(Clone, Debug, PartialEq)]
pub struct AstSelectionStatement {
    pub formula_objs: Vec<AstFormulaObject>,
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
            _ => panic!("The entity is not a Gid or a Str variant. 2")
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

            cube_def_tuple.member_roles.push(MemberRole::BaseMember {
                dim_role,
                member: dim_def_member,
            });
        }

        let mut formulas_map: HashMap<u64, AstFormulaObject> = HashMap::new();
        for frml_obj in &self.formula_objs {
            match frml_obj {
                AstFormulaObject::CustomFormulaMember(segments, _) => {
                    let frml_member_gid = segments.get_last_gid().unwrap();
                    formulas_map.insert(frml_member_gid, frml_obj.clone());
                }
            }
        }

        mdd::MultiDimensionalContext {
            cube,
            cube_def_tuple,
            grpc_client: grpc_cli,
            formulas_map,
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
        // 在解析AST时向函数调用栈深处传递的用于限定Cube切片范围的Tuple
        let mut slice_tuple = context.cube_def_tuple.clone();

        if let Some(slice) = &self.basic_slice {
            // mdx with `where statement`
            let where_tuple = match slice.materialize(&slice_tuple, context).await {
                MultiDimensionalEntity::TupleWrap(tuple) => tuple,
                _ => panic!("The entity is not a TupleWrap variant."),
            };
            slice_tuple = slice_tuple.merge(&where_tuple);
        }

        let axes_count = self.axes.len();

        for _ in 0..axes_count {
            for ast_axis in self.axes.iter() {
                let fiducial_tuple = ast_axis
                    .generate_fiducial_tuple(&slice_tuple, context)
                    .await;
                slice_tuple = slice_tuple.merge(&fiducial_tuple);
            }
        }

        let mut axes: Vec<mdd::Axis> = Vec::with_capacity(axes_count);

        for ast_axis in self.axes.iter() {
            let axis: mdd::Axis = ast_axis.translate_to_axis(&slice_tuple, context).await;
            axes.push(axis);
        }

        axes
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstFormulaObject {
    CustomFormulaMember(AstSegments, AstExpression),
    // CustomFormulaSet,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstExpression {
    pub terms: Vec<(char, AstTerm)>,
}

impl Materializable for AstExpression {
    async fn materialize(
        &self,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        let mut result = 0.0;
        let mut iter = self.terms.iter();
        if let Some((_, first_term)) = iter.next() {
            result += Box::pin(first_term.materialize(slice_tuple, context)) .await.cell_val();
        }

        for (op, term) in iter {
            let term_value = Box::pin(term.materialize(slice_tuple, context)) .await.cell_val();
            match *op {
                '+' => result += term_value,
                '-' => result -= term_value,
                _ => panic!("Invalid operator in AstExpression: {}", op),
            }
        }

        MultiDimensionalEntity::CellValue(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstFactory {
    FactoryNum(f64),
    FactorySegs(AstSegments),
    FactoryTuple(AstTuple),
    FactoryExp(AstExpression),
}

impl Materializable for AstFactory {
    async fn materialize(
        &self,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        match self {
            AstFactory::FactoryNum(num) => MultiDimensionalEntity::CellValue(*num),
            AstFactory::FactorySegs(segs) => {

                match segs.materialize(slice_tuple, context).await {
                    MultiDimensionalEntity::MemberRoleWrap(mr) => {

                        let ovc_tp = slice_tuple.merge(&Tuple {
                            member_roles: vec![mr]
                        });

                        let ovc = OlapVectorCoordinate {
                            member_roles: ovc_tp.member_roles,
                        };

                        let (_, values, _null_flags) = calculate(vec![ovc], context).await;

                        let measure_val = values.first().unwrap();

                        MultiDimensionalEntity::CellValue(*measure_val)
                    },
                    MultiDimensionalEntity::FormulaMemberWrap{dim_role_gid: _, exp} => {
                        exp.materialize(slice_tuple, context).await
                    },
                    _ => panic!("The entity is not a CellValue variant.")
                }
            },
            AstFactory::FactoryTuple(tuple) => {
                match tuple.materialize(slice_tuple, context).await {
                    MultiDimensionalEntity::TupleWrap(olap_tuple) => {
                        let ovc = OlapVectorCoordinate {
                            member_roles: slice_tuple.merge(&olap_tuple).member_roles,
                        };
                        let (_, values, _null_flags) = calculate(vec![ovc], context).await;
                        let measure_val = values.first().unwrap();
                        MultiDimensionalEntity::CellValue(*measure_val)
                    },
                    _ => panic!("The entity is not a TupleWrap variant.")
                }
            },
            AstFactory::FactoryExp(exp) => {
                let mde = exp.materialize(slice_tuple, context).await;
                if let MultiDimensionalEntity::CellValue(_) = mde {
                    mde
                } else {
                    panic!("The entity is not a CellValue variant.")
                }
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstTerm {
    pub factories: Vec<(char, AstFactory)>,
}

impl Materializable for AstTerm {
    async fn materialize(
        &self,
        _slice_tuple: &Tuple,
        _context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        let mut result = 0.0;
        let mut iter = self.factories.iter();
        if let Some((_, first_factory)) = iter.next() {
            result += first_factory.materialize(_slice_tuple, _context).await.cell_val();
        }

        for (op, factory) in iter {
            let factory_value = factory.materialize(_slice_tuple, _context).await.cell_val();
            match *op {
                '*' => result *= factory_value,
                '/' => {
                    if factory_value == 0.0 {
                        panic!("Divisor cannot be zero in AstTerm.");
                    }
                    result /= factory_value
                },
                _ => panic!("Invalid operator in AstTerm: {}", op),
            }
        }

        MultiDimensionalEntity::CellValue(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnParent {
    NoParam,
    HasParam(AstSegments),
}

impl AstMemberFnParent {
    async fn do_get_member(
        left_unique_param: Option<MultiDimensionalEntity>,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        if let MultiDimensionalEntity::MemberRoleWrap(mr) = left_unique_param.unwrap() {
            if let MemberRole::BaseMember {dim_role, member} = mr {
                if member.level < 1 {
                    return MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {dim_role, member});
                } else {
                    let obj = context.grpc_client.get_universal_olap_entity_by_gid(member.parent_gid).await.unwrap();
                    if let MultiDimensionalEntity::MemberWrap(member) = obj {
                        return MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember { 
                            dim_role,
                            member,
                        });
                    } else {
                        todo!()
                    }
                }
            }
        }
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFunction {
    Parent(AstMemberFnParent),
}

impl AstMemberFunction {
    pub async fn get_member(
        &self, 
        left_unique_param: Option<MultiDimensionalEntity>,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {

        match self {
            AstMemberFunction::Parent(AstMemberFnParent::NoParam) => {
                AstMemberFnParent::do_get_member(left_unique_param, context).await
            }
            AstMemberFunction::Parent(AstMemberFnParent::HasParam(_segs)) => {
                todo!("AstMemberFunction::get_member()")
            }
        }
    }
}
