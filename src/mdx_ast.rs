use core::panic;
use std::collections::HashMap;
// use std::vec;

use futures::future::BoxFuture;
// use std::pin::Pin;

use crate::mdd;
use crate::mdd::CellValue;
use crate::mdd::MultiDimensionalContext;
use crate::mdd::MultiDimensionalEntityLocator;
use crate::mdd::OlapVectorCoordinate;
use crate::mdd::{DimensionRole, Level, LevelRole};
use crate::mdd::{GidType, MemberRole, MultiDimensionalEntity, Set, Tuple};
use crate::olapmeta_grpc_client::GrpcClient;

use crate::meta_cache;

use crate::calcul::calculate;

pub trait Materializable {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut mdd::MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity>;
}

pub trait ToCellValue {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue>;
}

pub trait ToBoolValue {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool>;
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct AstCube {}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSeg {
    Gid(u64),
    Str(String),
    GidStr(u64, String),
    MemberFunction(AstMemberFunction),
    SetFunction(AstSetFunction),
    ExpFn(AstExpFunction),
    LevelFn(AstLevelFunction),
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
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut mdd::MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            match self {
                AstSeg::Gid(gid) => context.find_entity_by_gid(*gid).await,
                AstSeg::Str(seg_str) => context.find_entity_by_str(seg_str).await,
                AstSeg::GidStr(gid, _) => context.find_entity_by_gid(*gid).await,
                // MemberFunction(AstMemberFunction),
                AstSeg::MemberFunction(member_fn) => {
                    member_fn.get_member(None, slice_tuple, context).await
                }
                AstSeg::LevelFn(lv_fn) => {
                    let lv_role = lv_fn.get_level_role(None, slice_tuple, context).await;
                    MultiDimensionalEntity::LevelRole(lv_role)
                }
                AstSeg::ExpFn(exp_fn) => {
                    let exp_val = exp_fn.val(slice_tuple, context, None).await;
                    MultiDimensionalEntity::CellValue(exp_val)
                }
                _ => panic!("The entity is not a Gid or a Str variant. 1"),
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstSegments {
    pub segs: Vec<AstSeg>,
}

impl AstSegments {
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

impl Materializable for AstSegments {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut mdd::MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            let mut is_formula_member = false;

            let last_opt = self.get_last_gid();
            if let Some(last_gid) = last_opt {
                is_formula_member = GidType::entity_type(last_gid) == GidType::FormulaMember;
            }

            if is_formula_member {
                let dim_role_gid = self.get_first_gid().unwrap();
                let AstFormulaObject::CustomFormulaMember(_, exp) =
                    context.formulas_map.get(&last_opt.unwrap()).unwrap().clone();
                return MultiDimensionalEntity::FormulaMemberWrap { dim_role_gid, exp };
            }

            let ast_seg = self.segs.iter().next().unwrap();
            let head_entity: MultiDimensionalEntity =
                ast_seg.materialize(slice_tuple, context).await;

            if self.segs.len() == 1 {
                return head_entity;
            }

            match head_entity {
                MultiDimensionalEntity::DimensionRoleWrap(dim_role) => {
                    let tail_segs = AstSegments { segs: (self.segs[1..]).to_vec() };

                    dim_role.locate_entity(&tail_segs, slice_tuple, context).await
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
                _ => {
                    panic!("In method AstSegments::materialize(): head_entity is not a DimensionRoleWrap!");
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstTuple {
    SegsList(Vec<AstSegments>),
}

impl Materializable for AstTuple {
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut mdd::MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
            match self {
                AstTuple::SegsList(segs_list) => {
                    let mut member_roles: Vec<mdd::MemberRole> = Vec::new();
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
                    MultiDimensionalEntity::TupleWrap(mdd::Tuple { member_roles })
                }
            }
        })
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
                result = match tuples.iter().next().unwrap().materialize(slice_tuple, context).await
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
    fn materialize<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut mdd::MultiDimensionalContext,
    ) -> BoxFuture<'a, MultiDimensionalEntity> {
        Box::pin(async move {
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
        })
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
                        axis = mdd::Axis { set, pos_num: *pos as u32 };
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
            _ => panic!("The entity is not a Gid or a Str variant. 2"),
        }

        let mut cube_def_tuple = mdd::Tuple { member_roles: Vec::new() };

        let dimension_roles = grpc_cli.get_dimension_roles_by_cube_gid(cube.gid).await.unwrap();
        for dim_role in dimension_roles {
            let dim_def_member = grpc_cli
                .get_default_dimension_member_by_dimension_gid(dim_role.dimension_gid)
                .await
                .unwrap();

            cube_def_tuple
                .member_roles
                .push(MemberRole::BaseMember { dim_role, member: dim_def_member });
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

        let mut context = mdd::MultiDimensionalContext {
            cube,
            cube_def_tuple,
            where_tuple: None,
            query_slice_tuple: Tuple { member_roles: vec![] },
            grpc_client: grpc_cli,
            formulas_map,
        };

        let mut where_tuple: Option<Tuple> = None;
        if let Some(mdx_where) = &self.basic_slice {
            where_tuple =
                match mdx_where.materialize(&context.cube_def_tuple.clone(), &mut context).await {
                    MultiDimensionalEntity::TupleWrap(tuple) => Some(tuple),
                    _ => panic!("The entity is not a TupleWrap variant."),
                }
        };

        context.where_tuple = where_tuple;

        let mut query_slice_tuple = context.cube_def_tuple.clone();
        if let Some(where_tuple) = &context.where_tuple {
            query_slice_tuple = query_slice_tuple.merge(where_tuple);
        }
        context.query_slice_tuple = query_slice_tuple;

        context
    }

    async fn fetch_cube_by_gid(&self, grpc_cli: &mut GrpcClient, gid: u64) -> mdd::Cube {
        match grpc_cli.get_cube_by_gid(gid).await {
            Ok(response) => response
                .cube_meta
                .map(|meta| mdd::Cube { gid: meta.gid, name: meta.name })
                .unwrap_or_else(|| {
                    println!("Error fetching Cube by GID: CubeMeta is None");
                    mdd::Cube { gid: 0, name: String::from(">>> No cube found <<<") }
                }),
            Err(e) => {
                println!("Error fetching Cube by GID: {}", e);
                mdd::Cube { gid: 0, name: String::from(">>> No cube found <<<") }
            }
        }
    }

    async fn fetch_cube_by_name(&self, grpc_cli: &mut GrpcClient, name: &str) -> mdd::Cube {
        match grpc_cli.get_cube_by_name(name.to_string()).await {
            Ok(response) => {
                println!("Received Cube by Name: {:?}", response);
                response
                    .cube_meta
                    .map(|meta| mdd::Cube { gid: meta.gid, name: meta.name })
                    .unwrap_or_else(|| {
                        println!("Error fetching Cube by Name: CubeMeta is None");
                        mdd::Cube { gid: 0, name: String::from(">>> No cube found <<<") }
                    })
            }
            Err(e) => {
                println!("Error fetching Cube by Name: {}", e);
                mdd::Cube { gid: 0, name: String::from(">>> No cube found <<<") }
            }
        }
    }

    pub async fn build_axes(&self, context: &mut mdd::MultiDimensionalContext) -> Vec<mdd::Axis> {
        let mut slice_tuple = context.query_slice_tuple.clone();

        let axes_count = self.axes.len();

        for _ in 0..axes_count {
            for ast_axis in self.axes.iter() {
                let fiducial_tuple = ast_axis.generate_fiducial_tuple(&slice_tuple, context).await;
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

impl ToCellValue for AstExpression {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            let mut result = CellValue::Invalid;
            for (index, (op, term)) in self.terms.iter().enumerate() {
                if index == 0 {
                    result = Box::pin(term.val(slice_tuple, context, None)).await;
                    continue;
                }

                let term_value = Box::pin(term.val(slice_tuple, context, None)).await;
                match *op {
                    '+' => result = result + term_value,
                    '-' => result = result - term_value,
                    _ => panic!("Invalid operator in AstExpression: {}", op),
                }
            }
            result
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstFactory {
    FactoryNum(f64),
    FactoryStr(String),
    FactorySegs(AstSegments),
    FactoryTuple(AstTuple),
    FactoryExp(AstExpression),
}

impl ToCellValue for AstFactory {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            match self {
                AstFactory::FactoryNum(num) => CellValue::Double(*num),
                AstFactory::FactoryStr(str) => CellValue::Str(String::from(str)),
                AstFactory::FactorySegs(segs) => match segs.materialize(slice_tuple, context).await
                {
                    MultiDimensionalEntity::MemberRoleWrap(mr) => {
                        let ovc_tp = slice_tuple.merge(&Tuple { member_roles: vec![mr] });

                        let ovc = OlapVectorCoordinate { member_roles: ovc_tp.member_roles };

                        let cell_values = calculate(vec![ovc], context).await;
                        cell_values.first().unwrap().clone()
                    }
                    MultiDimensionalEntity::FormulaMemberWrap { dim_role_gid: _, exp } => {
                        exp.val(slice_tuple, context, None).await
                    }
                    MultiDimensionalEntity::ExpFn(exp_fn) => {
                        exp_fn.val(slice_tuple, context, None).await
                    }
                    MultiDimensionalEntity::CellValue(cell_value) => cell_value.clone(),
                    _ => panic!("The entity is not a CellValue variant."),
                },
                AstFactory::FactoryTuple(tuple) => {
                    match tuple.materialize(slice_tuple, context).await {
                        MultiDimensionalEntity::TupleWrap(olap_tuple) => {
                            let ovc = OlapVectorCoordinate {
                                member_roles: slice_tuple.merge(&olap_tuple).member_roles,
                            };
                            let cell_values = calculate(vec![ovc], context).await;
                            cell_values.first().unwrap().clone()
                        }
                        _ => panic!("The entity is not a TupleWrap variant."),
                    }
                }
                AstFactory::FactoryExp(exp) => exp.val(slice_tuple, context, None).await,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstTerm {
    pub factories: Vec<(char, AstFactory)>,
}

impl ToCellValue for AstTerm {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            let mut result = CellValue::Invalid;
            for (index, (op, factory)) in self.factories.iter().enumerate() {
                if index == 0 {
                    result = factory.val(slice_tuple, context, None).await;
                    continue;
                }

                let factory_value = factory.val(slice_tuple, context, None).await;
                match *op {
                    '*' => result = result * factory_value,
                    '/' => result = result / factory_value,
                    _ => panic!("Invalid operator in AstTerm: {}", op),
                }
            }
            result
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnClosingPeriod {
    NoParam,
    OneParam(AstSegments),
    TwoParams(AstSegments, AstSegments),
}

impl AstMemberFnClosingPeriod {
    async fn do_get_member(
        left_outer_param: Option<MultiDimensionalEntity>,
        level_param: Option<&AstSegments>,
        member_param: Option<&AstSegments>,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match (left_outer_param, level_param, member_param) {
            (None, Some(lv_segs), None) => {
                let olap_obj = lv_segs.materialize(slice_tuple, context).await;
                if let MultiDimensionalEntity::LevelRole(lv_role) = olap_obj {
                    MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role: lv_role.dim_role.clone(),
                        member: meta_cache::get_member_by_gid(lv_role.level.closing_period_gid),
                    })
                } else {
                    panic!("[850BHE] The entity is not a LevelRole variant.");
                }
            }
            _ => {
                panic!("Invalid parameter combination. Only level_param should be Some, and left_outer_param and member_param should be None.");
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstMemberFnOpeningPeriod {
    NoParam,
    OneParam(AstSegments),
    TwoParams(AstSegments, AstSegments),
}

impl AstMemberFnOpeningPeriod {
    async fn do_get_member(
        left_outer_param: Option<MultiDimensionalEntity>,
        level_param: Option<&AstSegments>,
        member_param: Option<&AstSegments>,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match (left_outer_param, level_param, member_param) {
            (None, Some(lv_segs), None) => {
                let olap_obj = lv_segs.materialize(slice_tuple, context).await;
                if let MultiDimensionalEntity::LevelRole(lv_role) = olap_obj {
                    MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role: lv_role.dim_role.clone(),
                        member: meta_cache::get_member_by_gid(lv_role.level.opening_period_gid),
                    })
                } else {
                    panic!("[833BHE] The entity is not a LevelRole variant.");
                }
            }
            _ => {
                panic!("[hsb778] Invalid parameter combination. Only level_param should be Some, and left_outer_param and member_param should be None.");
            }
        }
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
            if let MemberRole::BaseMember { dim_role, member } = mr {
                if member.level < 1 {
                    return MultiDimensionalEntity::MemberRoleWrap(MemberRole::BaseMember {
                        dim_role,
                        member,
                    });
                } else {
                    let obj = context
                        .grpc_client
                        .get_universal_olap_entity_by_gid(member.parent_gid)
                        .await
                        .unwrap();
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
    ClosingPeriod(AstMemberFnClosingPeriod),
    OpeningPeriod(AstMemberFnOpeningPeriod),
}

impl AstMemberFunction {
    pub async fn get_member(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> MultiDimensionalEntity {
        match self {
            // parent()
            AstMemberFunction::Parent(AstMemberFnParent::NoParam) => {
                AstMemberFnParent::do_get_member(left_outer_param, context).await
            }
            AstMemberFunction::Parent(AstMemberFnParent::HasParam(_segs)) => {
                todo!("AstMemberFunction::get_member()")
            }
            // ClosingPeriod()
            AstMemberFunction::ClosingPeriod(AstMemberFnClosingPeriod::NoParam) => {
                AstMemberFnClosingPeriod::do_get_member(
                    left_outer_param,
                    None,
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::ClosingPeriod(AstMemberFnClosingPeriod::OneParam(level_segs)) => {
                AstMemberFnClosingPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::ClosingPeriod(AstMemberFnClosingPeriod::TwoParams(
                level_segs,
                member_segs,
            )) => {
                AstMemberFnClosingPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    Some(member_segs),
                    slice_tuple,
                    context,
                )
                .await
            }
            // OpeningPeriod()
            AstMemberFunction::OpeningPeriod(AstMemberFnOpeningPeriod::NoParam) => {
                AstMemberFnOpeningPeriod::do_get_member(
                    left_outer_param,
                    None,
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::OpeningPeriod(AstMemberFnOpeningPeriod::OneParam(level_segs)) => {
                AstMemberFnOpeningPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    None,
                    slice_tuple,
                    context,
                )
                .await
            }
            AstMemberFunction::OpeningPeriod(AstMemberFnOpeningPeriod::TwoParams(
                level_segs,
                member_segs,
            )) => {
                AstMemberFnOpeningPeriod::do_get_member(
                    left_outer_param,
                    Some(level_segs),
                    Some(member_segs),
                    slice_tuple,
                    context,
                )
                .await
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFunction {
    Level(AstLevelFnLevel),
    Levels(AstLevelFnLevels),
}

impl AstLevelFunction {
    pub async fn get_level_role(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> LevelRole {
        match self {
            AstLevelFunction::Level(fn_level) => {
                fn_level.get_level_role(left_outer_param, slice_tuple, context).await
            }
            AstLevelFunction::Levels(fn_levels) => {
                fn_levels.get_level_role(left_outer_param, slice_tuple, context).await
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstLevelFnLevel {
    NoParam,
    OneParam(AstSegments),
}

impl AstLevelFnLevel {
    fn do_get_level_role(&self, mr: MemberRole) -> LevelRole {
        if let MemberRole::BaseMember { dim_role, member } = mr {
            LevelRole::new(dim_role, meta_cache::get_level_by_gid(member.level_gid))
        } else {
            panic!("[003BHE] The entity is not a MemberRole variant.");
        }
    }

    async fn get_level_role(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> LevelRole {
        if let Some(MultiDimensionalEntity::MemberRoleWrap(mr)) = left_outer_param {
            return self.do_get_level_role(mr);
        }

        if let AstLevelFnLevel::OneParam(ast_segs) = self {
            if let MultiDimensionalEntity::MemberRoleWrap(mr) =
                ast_segs.materialize(slice_tuple, context).await
            {
                return self.do_get_level_role(mr);
            }
        }

        panic!("[klu704] AstLevelFnLevel::do_get_level_role()");
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstLevelFnLevels {
    dim_segs: Option<AstSegments>,
    idx_exp: AstExpression,
}

impl AstLevelFnLevels {
    pub fn new(dim_segs: Option<AstSegments>, idx_exp: AstExpression) -> Self {
        Self { dim_segs, idx_exp }
    }

    async fn get_level_role(
        &self,
        left_outer_param: Option<MultiDimensionalEntity>,
        slice_tuple: &Tuple,
        context: &mut MultiDimensionalContext,
    ) -> LevelRole {
        let mut param_dim_role: Option<DimensionRole> = None;
        let mut def_hierarchy_gid = 0;

        if let Some(MultiDimensionalEntity::DimensionRoleWrap(dr)) = left_outer_param {
            def_hierarchy_gid = dr.default_hierarchy_gid;
            param_dim_role = Some(dr);
        } else if let Some(ast_segs) = &self.dim_segs {
            if let MultiDimensionalEntity::DimensionRoleWrap(dr) =
                ast_segs.materialize(slice_tuple, context).await
            {
                def_hierarchy_gid = dr.default_hierarchy_gid;
                param_dim_role = Some(dr);
            } else {
                panic!("[003BHE] The entity is not a DimensionRole variant.");
            }
        }

        if let None = param_dim_role {
            panic!("[033BHE] The entity is not a DimensionRole variant.");
        }

        let param_dim_role = param_dim_role.unwrap();

        let cell_val = self.idx_exp.val(slice_tuple, context, None).await;
        if let CellValue::Double(idx) = cell_val {
            let lv_val = idx as u32;

            let olap_obj_level: Level = meta_cache::get_hierarchy_level(def_hierarchy_gid, lv_val);
            LevelRole::new(param_dim_role, olap_obj_level)
        } else {
            panic!("[klu704] AstLevelFnLevel::do_get_level_role()");
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFnChildren {
    NoParam,
    InnerParam(AstSegments),
}

impl AstSetFnChildren {
    async fn do_get_set(
        left_unique_param: Option<MultiDimensionalEntity>,
        context: &mut MultiDimensionalContext,
    ) -> Set {
        if let MultiDimensionalEntity::MemberRoleWrap(mr) = left_unique_param.unwrap() {
            if let MemberRole::BaseMember { dim_role, member } = mr {
                let children =
                    context.grpc_client.get_child_members_by_gid(member.gid).await.unwrap();

                let tuples: Vec<Tuple> = children
                    .into_iter()
                    .map(|child| Tuple {
                        member_roles: vec![MemberRole::BaseMember {
                            dim_role: dim_role.clone(),
                            member: child,
                        }],
                    })
                    .collect();

                return Set { tuples };
            }
        }

        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstSetFunction {
    Children(AstSetFnChildren),
}

impl AstSetFunction {
    pub async fn get_set(
        &self,
        left_unique_param: Option<MultiDimensionalEntity>,
        context: &mut MultiDimensionalContext,
    ) -> Set {
        match self {
            AstSetFunction::Children(AstSetFnChildren::NoParam) => {
                AstSetFnChildren::do_get_set(left_unique_param, context).await
            }
            AstSetFunction::Children(AstSetFnChildren::InnerParam(_segs)) => {
                todo!("AstSetFunction::get_set()")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFunction {
    Avg(AstExpFnAvg),
    Count(AstExpFnCount),
    IIf(AstExpFnIIf),
    Name(AstExpFnName),
}

impl ToCellValue for AstExpFunction {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
        outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            match self {
                AstExpFunction::Avg(avg_fn) => avg_fn.val(slice_tuple, context, None).await,
                AstExpFunction::Count(count_fn) => count_fn.val(slice_tuple, context, None).await,
                AstExpFunction::IIf(iif_fn) => iif_fn.val(slice_tuple, context, None).await,
                AstExpFunction::Name(name_fn) => {
                    if let Some(olap_obj) = outer_param {
                        let name_fn = AstExpFnName::OuterParam(Box::new(olap_obj));
                        name_fn.val(slice_tuple, context, None).await
                    } else {
                        name_fn.val(slice_tuple, context, None).await
                    }
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnName {
    NoParam,
    InnerParam(AstSegments),
    OuterParam(Box<MultiDimensionalEntity>),
}

impl ToCellValue for AstExpFnName {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            //  CellValue::Str("avg函数有待实现".to_string())
            match self {
                AstExpFnName::InnerParam(segs) => {
                    let olap_obj = segs.materialize(slice_tuple, context).await;
                    if let MultiDimensionalEntity::MemberRoleWrap(member_role) = olap_obj {
                        match member_role {
                            MemberRole::BaseMember { member, .. } => {
                                CellValue::Str(member.name.clone())
                            }
                            _ => CellValue::Str("name函数参数错误".to_string()),
                        }
                    } else {
                        CellValue::Str("name函数参数错误".to_string())
                    }
                }
                AstExpFnName::OuterParam(entity) => {
                    if let MultiDimensionalEntity::MemberRoleWrap(member_role) = entity.as_ref() {
                        match member_role {
                            MemberRole::BaseMember { member, .. } => {
                                CellValue::Str(member.name.clone())
                            }
                            _ => CellValue::Str("name函数参数错误".to_string()),
                        }
                    } else {
                        CellValue::Str("name函数参数错误".to_string())
                    }
                }
                _ => panic!("[dsuc-0-8492] AstExpFnName::val()"),
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnAvg {
    NoParam,
    InnerParam(AstSet),
    OuterParam(Set),
}

impl ToCellValue for AstExpFnAvg {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a Tuple,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move { CellValue::Str("avg函数有待实现".to_string()) })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpFnCount {
    NoParam,
    InnerParam(AstSet),
    OuterParam(Set),
}

impl ToCellValue for AstExpFnCount {
    fn val<'a>(
        &'a self,
        _slice_tuple: &'a Tuple,
        _context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            let set = match self {
                AstExpFnCount::InnerParam(_set) => {
                    todo!("AstExpFnCount::val()")
                }
                AstExpFnCount::OuterParam(set) => set,
                _ => panic!("AstExpFnCount::val()"),
            };

            CellValue::Double(set.tuples.len() as f64)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstExpFnIIf {
    pub bool_exp: AstBoolExp,
    pub exp_t: AstExpression,
    pub exp_f: AstExpression,
}

impl ToCellValue for AstExpFnIIf {
    fn val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
        _outer_param: Option<MultiDimensionalEntity>,
    ) -> BoxFuture<'a, CellValue> {
        Box::pin(async move {
            let bool_val = self.bool_exp.bool_val(slice_tuple, context).await;
            if bool_val {
                self.exp_t.val(slice_tuple, context, None).await
            } else {
                self.exp_f.val(slice_tuple, context, None).await
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolExp {
    BoolTerm(AstBoolTerm),
    NotBoolTerm(AstBoolTerm),
    BoolExpOrBoolTerm(Box<AstBoolExp>, AstBoolTerm),
}

impl ToBoolValue for AstBoolExp {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolExp::BoolTerm(bool_term) => bool_term.bool_val(slice_tuple, context).await,
                AstBoolExp::NotBoolTerm(bool_term) => {
                    !bool_term.bool_val(slice_tuple, context).await
                }
                AstBoolExp::BoolExpOrBoolTerm(bool_exp, bool_term) => {
                    let exp_bool = bool_exp.bool_val(slice_tuple, context).await;
                    if exp_bool {
                        true
                    } else {
                        bool_term.bool_val(slice_tuple, context).await
                    }
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolTerm {
    BoolFactory(AstBoolFactory),
    BoolTermAndBoolFactory(Box<AstBoolTerm>, AstBoolFactory),
}

impl ToBoolValue for AstBoolTerm {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolTerm::BoolFactory(bool_factory) => {
                    bool_factory.bool_val(slice_tuple, context).await
                }
                AstBoolTerm::BoolTermAndBoolFactory(bool_term, bool_factory) => {
                    let term_bool = bool_term.bool_val(slice_tuple, context).await;
                    if term_bool {
                        bool_factory.bool_val(slice_tuple, context).await
                    } else {
                        false
                    }
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolFactory {
    ExpressionComparesAnother(AstExpression, String, AstExpression),
    BoolExp(Box<AstBoolExp>),
    BoolFn(AstBoolFunction),
}

impl ToBoolValue for AstBoolFactory {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolFactory::ExpressionComparesAnother(exp1, op, exp2) => {
                    let val1 = exp1.val(slice_tuple, context, None).await;
                    let val2 = exp2.val(slice_tuple, context, None).await;
                    val1.logical_cmp(op, &val2)
                }
                AstBoolFactory::BoolExp(bool_exp) => bool_exp.bool_val(slice_tuple, context).await,
                AstBoolFactory::BoolFn(bool_fn) => bool_fn.bool_val(slice_tuple, context).await,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstBoolFunction {
    IsLeaf(AstBoolFnIsLeaf),
}

impl ToBoolValue for AstBoolFunction {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self {
                AstBoolFunction::IsLeaf(is_leaf_fn) => {
                    is_leaf_fn.bool_val(slice_tuple, context).await
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstBoolFnIsLeaf {
    pub member_segs: AstSegments,
}

impl AstBoolFnIsLeaf {
    pub fn new(member_segs: AstSegments) -> Self {
        Self { member_segs }
    }
}

impl ToBoolValue for AstBoolFnIsLeaf {
    fn bool_val<'a>(
        &'a self,
        slice_tuple: &'a Tuple,
        context: &'a mut MultiDimensionalContext,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            let olap_obj = self.member_segs.materialize(slice_tuple, context).await;
            if let MultiDimensionalEntity::MemberRoleWrap(member_role) = olap_obj {
                match member_role {
                    MemberRole::BaseMember { member, .. } => member.leaf,
                    _ => true,
                }
            } else {
                panic!("[hsju6679] The entity is not a MemberRole variant.");
            }
        })
    }
}
