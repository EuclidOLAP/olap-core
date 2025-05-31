// calculation module

use crate::mdx_ast::ToCellValue;

use crate::mdd::OlapVectorCoordinate;
use crate::mdd::{CellValue, MultiDimensionalContext};
use crate::mdd::{MemberRole, Tuple};

use crate::agg_service_client::basic_aggregates;

pub async fn calculate(
    vs: Vec<OlapVectorCoordinate>,
    context: &mut MultiDimensionalContext,
) -> Vec<CellValue> {
    // Base OlapVectorCoordinates and Formula OlapVectorCoordinates
    // 分别存储索引和坐标数据
    let mut base_indices: Vec<usize> = Vec::new();
    let mut frml_indices: Vec<usize> = Vec::new();
    let mut base_cords: Vec<OlapVectorCoordinate> = Vec::new();
    let mut frml_cords: Vec<OlapVectorCoordinate> = Vec::new();

    'outside: for (idx, cord) in vs.into_iter().enumerate() {
        for mr in &cord.member_roles {
            if let MemberRole::FormulaMember { dim_role_gid: _, exp: _ } = mr {
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
        .map(
            |((val, flag), idx)| {
                if flag {
                    (CellValue::Null, idx)
                } else {
                    (CellValue::Double(val), idx)
                }
            },
        )
        .collect();

    let calc_cell_vals = calculate_formula_vectors(frml_cords, context).await;
    let calc_combined: Vec<(CellValue, usize)> =
        calc_cell_vals.into_iter().zip(frml_indices.into_iter()).collect();

    let mut cells_indices = base_combined;
    cells_indices.extend(calc_combined);
    cells_indices.sort_by(|a, b| a.1.cmp(&b.1));

    cells_indices.into_iter().map(|(cell_val, _)| cell_val).collect()
}

async fn calculate_formula_vectors(
    coordinates: Vec<OlapVectorCoordinate>,
    context: &mut MultiDimensionalContext,
) -> Vec<CellValue> {
    let mut values: Vec<CellValue> = Vec::new();

    'outer_loop: for cord in coordinates {
        for mr in cord.member_roles.iter().rev() {
            if let MemberRole::FormulaMember { dim_role_gid: _, exp } = mr {
                // VCE C langueage
                // Expression *exp = mr->member_formula->exp;
                // Expression_evaluate(md_ctx, exp, cube, cal_tp, gd);

                let exp = exp.clone();

                values
                    .push(exp.val(&Tuple { member_roles: cord.member_roles }, context, None).await);

                continue 'outer_loop;
            }
        }
        panic!("[calculate_formula_vectors()] - It's not a formula member role: ______");
    }

    values
    // (context.cube.gid, doubles, bools)
}
