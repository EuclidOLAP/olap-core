// calculation module

use crate::exmdx::ast::ToVectorValue;

use crate::exmdx::mdd::TupleVector;
use crate::mdd::MemberRole;
use crate::mdd::{VectorValue, MultiDimensionalContext};

use crate::agg_service_client::basic_aggregates;

pub async fn calculate(
    vs: Vec<TupleVector>,
    context: &mut MultiDimensionalContext,
) -> Vec<VectorValue> {
    // Base OlapVectorCoordinates and Formula OlapVectorCoordinates
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
    let base_combined: Vec<(VectorValue, usize)> = base_vals
        .into_iter()
        .zip(base_null_flags.into_iter())
        .zip(base_indices.into_iter())
        .map(|((val, flag), idx)| {
            if flag {
                (VectorValue::Null, idx)
            } else {
                (VectorValue::Double(val), idx)
            }
        })
        .collect();

    let calc_cell_vals = calculate_formula_vectors(frml_cords, context).await;
    let calc_combined: Vec<(VectorValue, usize)> = calc_cell_vals
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
) -> Vec<VectorValue> {
    let mut values: Vec<VectorValue> = Vec::new();

    'outer_loop: for cord in coordinates {
        for mr in cord.member_roles.iter().rev() {
            if let MemberRole::FormulaMember { dim_role_gid, exp } = mr {
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
