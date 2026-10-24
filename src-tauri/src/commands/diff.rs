use crate::db::DbState;
use rusqlite::{params, OptionalExtension, types::Type};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlanDiff {
    pub field: String,
    pub diff_type: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDiffResult {
    pub version_a_id: String,
    pub version_b_id: String,
    pub diffs: Vec<PlanDiff>,
    pub similarity: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffPlanVersionsInput {
    pub version_id_a: String,
    pub version_id_b: String,
}

fn row_to_plan_version_raw(row: &rusqlite::Row<'_>) -> rusqlite::Result<(String, String, Value)> {
    let id: String = row.get(0)?;
    let agent_id: String = row.get(1)?;
    let spec_str: String = row.get(3)?;
    let spec_json: Value = serde_json::from_str(&spec_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(e)))?;
    Ok((id, agent_id, spec_json))
}

fn compute_diff(
    a: &Value,
    b: &Value,
    path: &str,
    diffs: &mut Vec<PlanDiff>,
) {
    match (a, b) {
        (Value::Object(oa), Value::Object(ob)) => {
            let all_keys: std::collections::BTreeSet<&String> =
                oa.keys().chain(ob.keys()).collect();
            for key in all_keys {
                let field = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                match (oa.get(key), ob.get(key)) {
                    (Some(va), Some(vb)) => {
                        compute_diff(va, vb, &field, diffs);
                    }
                    (Some(va), None) => {
                        diffs.push(PlanDiff {
                            field,
                            diff_type: "removed".to_string(),
                            before: Some(va.clone()),
                            after: None,
                        });
                    }
                    (None, Some(vb)) => {
                        diffs.push(PlanDiff {
                            field,
                            diff_type: "added".to_string(),
                            before: None,
                            after: Some(vb.clone()),
                        });
                    }
                    (None, None) => {}
                }
            }
        }
        (Value::Array(va), Value::Array(vb)) => {
            let max_len = va.len().max(vb.len());
            for i in 0..max_len {
                let field = format!("{}[{}]", path, i);
                match (va.get(i), vb.get(i)) {
                    (Some(va_item), Some(vb_item)) => {
                        compute_diff(va_item, vb_item, &field, diffs);
                    }
                    (Some(va_item), None) => {
                        diffs.push(PlanDiff {
                            field,
                            diff_type: "removed".to_string(),
                            before: Some(va_item.clone()),
                            after: None,
                        });
                    }
                    (None, Some(vb_item)) => {
                        diffs.push(PlanDiff {
                            field,
                            diff_type: "added".to_string(),
                            before: None,
                            after: Some(vb_item.clone()),
                        });
                    }
                    (None, None) => {}
                }
            }
        }
        (va, vb) => {
            if va != vb {
                diffs.push(PlanDiff {
                    field: path.to_string(),
                    diff_type: "changed".to_string(),
                    before: Some(va.clone()),
                    after: Some(vb.clone()),
                });
            }
        }
    }
}

#[tauri::command]
pub fn diff_plan_versions(
    state: State<'_, DbState>,
    input: DiffPlanVersionsInput,
) -> Result<PlanDiffResult, String> {
    let conn = &*state.conn.lock().unwrap();

    let version_a: Option<(String, String, Value)> = conn
        .prepare("SELECT id, agent_id, spec_json FROM plan_versions WHERE id = ?1")
        .map_err(|e| e.to_string())?
        .query_row(params![input.version_id_a], row_to_plan_version_raw)
        .optional()
        .map_err(|e| e.to_string())?;

    let version_b: Option<(String, String, Value)> = conn
        .prepare("SELECT id, agent_id, spec_json FROM plan_versions WHERE id = ?1")
        .map_err(|e| e.to_string())?
        .query_row(params![input.version_id_b], row_to_plan_version_raw)
        .optional()
        .map_err(|e| e.to_string())?;

    let (id_a, _agent_a, spec_a) = version_a
        .ok_or_else(|| format!("Plan version not found: {}", input.version_id_a))?;
    let (id_b, _agent_b, spec_b) = version_b
        .ok_or_else(|| format!("Plan version not found: {}", input.version_id_b))?;

    let mut diffs: Vec<PlanDiff> = Vec::new();
    compute_diff(&spec_a, &spec_b, "", &mut diffs);

    let total_fields = count_fields(&spec_a) + count_fields(&spec_b);
    let similarity = if total_fields == 0 {
        1.0
    } else {
        let changed_count = diffs.len() as f64;
        1.0 - (changed_count / total_fields as f64)
    };

    Ok(PlanDiffResult {
        version_a_id: id_a,
        version_b_id: id_b,
        diffs,
        similarity: similarity.max(0.0),
    })
}

fn count_fields(value: &Value) -> usize {
    match value {
        Value::Object(map) => map.values().map(count_fields).sum::<usize>() + map.len(),
        Value::Array(arr) => arr.iter().map(count_fields).sum::<usize>() + arr.len(),
        _ => 1,
    }
}
