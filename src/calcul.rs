// calculation module

use crate::mdx_ast::ToCellValue;

use crate::exmdx::mdd::TupleVector;
use crate::mdd::MemberRole;
use crate::mdd::{CellValue, MultiDimensionalContext};

use crate::agg_service_client::basic_aggregates;

pub async fn calculate(
    vs: Vec<TupleVector>,
    context: &mut MultiDimensionalContext,
) -> Vec<CellValue> {
    // Base OlapVectorCoordinates and Formula OlapVectorCoordinates
    // 分别存储索引和坐标数据
    let mut base_indices: Vec<usize> = Vec::new();
    let mut frml_indices: Vec<usize> = Vec::new();
    let mut base_cords: Vec<TupleVector> = Vec::new();
    let mut frml_cords: Vec<TupleVector> = Vec::new();

    'outside: for (idx, cord) in vs.into_iter().enumerate() {
        for mr in &cord.member_roles {
            if let MemberRole::FormulaMember {
                dim_role_gid: _,
                exp: _,
            } = mr
            {
                frml_indices.push(idx);
                frml_cords.push(cord);
                continue 'outside;
            }
        }
        base_indices.push(idx);
        base_cords.push(cord);
    }

    let (_cube_gid, base_vals, base_null_flags) = basic_aggregates(base_cords, context).await;

    // let combined: Vec<(f64, bool)> = base_vals.into_iter() .zip(base_null_flags.into_iter()) .collect();
    let base_combined: Vec<(CellValue, usize)> = base_vals
        .into_iter()
        .zip(base_null_flags.into_iter())
        .zip(base_indices.into_iter())
        .map(|((val, flag), idx)| {
            if flag {
                (CellValue::Null, idx)
            } else {
                (CellValue::Double(val), idx)
            }
        })
        .collect();

    let calc_cell_vals = calculate_formula_vectors(frml_cords, context).await;
    let calc_combined: Vec<(CellValue, usize)> = calc_cell_vals
        .into_iter()
        .zip(frml_indices.into_iter())
        .collect();

    let mut cells_indices = base_combined;
    cells_indices.extend(calc_combined);
    cells_indices.sort_by(|a, b| a.1.cmp(&b.1));

    cells_indices
        .into_iter()
        .map(|(cell_val, _)| cell_val)
        .collect()
}

async fn calculate_formula_vectors(
    coordinates: Vec<TupleVector>,
    context: &mut MultiDimensionalContext,
) -> Vec<CellValue> {
    let mut values: Vec<CellValue> = Vec::new();

    'outer_loop: for cord in coordinates {
        for mr in cord.member_roles.iter().rev() {
            // if let MemberRole::FormulaMember { dim_role_gid: _, exp } = mr {
            //     // VCE C langueage
            //     // Expression *exp = mr->member_formula->exp;
            //     // Expression_evaluate(md_ctx, exp, cube, cal_tp, gd);

            //     let exp = exp.clone();
            //     values
            //         .push(exp.val(&Tuple { member_roles: cord.member_roles }, context, None).await);
            //     continue 'outer_loop;
            // }

            if let MemberRole::FormulaMember { dim_role_gid, exp } = mr {
                // VCE C langueage
                // Expression *exp = mr->member_formula->exp;
                // Expression_evaluate(md_ctx, exp, cube, cal_tp, gd);

                // todo dim_role_gid是有用的，需要获得对应的默认维度成员角色，将其嵌入到cord中，形成新的slice_tuple
                // 然后调用AstExpression::val()方法，计算出结果值
                let dim_role = context
                    .grpc_client
                    .get_dimension_role_by_gid(*dim_role_gid)
                    .await
                    .unwrap();
                let member = context
                    .grpc_client
                    .get_default_dimension_member_by_dimension_gid(dim_role.dimension_gid)
                    .await
                    .unwrap();

                let member_role = MemberRole::BaseMember { dim_role, member };
                let one_mr_tup = TupleVector {
                    member_roles: vec![member_role],
                };
                let slice_tuple = TupleVector {
                    member_roles: cord.member_roles.clone(),
                }
                .merge(&one_mr_tup);

                // todo 同一个表达式应该在不同的上下文下计算得不同的值，貌似不需要clone啊
                let exp = exp.clone();

                values.push(exp.val(&slice_tuple, context, None).await);

                continue 'outer_loop;
            }
        }
        panic!("[calculate_formula_vectors()] - It's not a formula member role: ______");
    }

    values
    // (context.cube.gid, doubles, bools)
}
