// calculation module

use crate::mdx_ast::ToCellValue;

use crate::mdd::OlapVectorCoordinate;
use crate::mdd::{CellValue, MultiDimensionalContext};
use crate::mdd::{MemberRole, Tuple};

use crate::agg_service_client::basic_aggregates;

pub async fn calculate(
    vs: Vec<OlapVectorCoordinate>,
    context: &mut MultiDimensionalContext,
) -> (u64, Vec<f64>, Vec<bool>) {
    // Base OlapVectorCoordinates and Formula OlapVectorCoordinates
    // 分别存储索引和坐标数据
    let mut base_indices: Vec<usize> = Vec::new();
    let mut frml_indices: Vec<usize> = Vec::new();
    let mut base_cords: Vec<OlapVectorCoordinate> = Vec::new();
    let mut frml_cords: Vec<OlapVectorCoordinate> = Vec::new();

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

    let (cube_gid, base_vals, base_null_flags) = basic_aggregates(base_cords, context).await;
    let (_, frml_vals, frml_null_flags) = calculate_formula_vectors(frml_cords, context).await;

    // 初始化最终的合并结果，确保索引对应数据不乱序
    let mut merged_vals = vec![0.0; base_indices.len() + frml_indices.len()];
    let mut merged_null_flags = vec![false; base_indices.len() + frml_indices.len()];

    // 填充基本数据
    for (idx, value) in base_indices.iter().zip(base_vals.iter()) {
        merged_vals[*idx] = *value;
    }
    for (idx, flag) in base_indices.iter().zip(base_null_flags.iter()) {
        merged_null_flags[*idx] = *flag;
    }

    // 填充公式数据
    for (idx, value) in frml_indices.iter().zip(frml_vals.iter()) {
        merged_vals[*idx] = *value;
    }
    for (idx, flag) in frml_indices.iter().zip(frml_null_flags.iter()) {
        merged_null_flags[*idx] = *flag;
    }

    (cube_gid, merged_vals, merged_null_flags)
}

async fn calculate_formula_vectors(
    coordinates: Vec<OlapVectorCoordinate>,
    context: &mut MultiDimensionalContext,
) -> (u64, Vec<f64>, Vec<bool>) {
    let mut doubles: Vec<f64> = Vec::new();
    let mut bools: Vec<bool> = Vec::new();

    'outer_loop: for cord in coordinates {
        for mr in cord.member_roles.iter().rev() {
            if let MemberRole::FormulaMember {
                dim_role_gid: _,
                exp,
            } = mr
            {
                // VCE C langueage
                // Expression *exp = mr->member_formula->exp;
                // Expression_evaluate(md_ctx, exp, cube, cal_tp, gd);

                let exp = exp.clone();
                let cell_val = exp
                    .val(
                        &Tuple {
                            member_roles: cord.member_roles,
                        },
                        context,
                    )
                    .await;

                if let CellValue::Double(val) = cell_val {
                    doubles.push(val);
                    bools.push(false);
                } else {
                    doubles.push(-0.999999999);
                    bools.push(false);
                }

                continue 'outer_loop;
            }
        }
        panic!("[calculate_formula_vectors()] - It's not a formula member role: ______");
    }

    (context.cube.gid, doubles, bools)
}
