use serde_json::Value;

/// Agent plan schema understood by this build.
pub const PLAN_VERSION: i64 = 2;

/// Structural validation for an agent specification.
///
/// Accepts schema v2 (`triggers` / `results` / `chains`) and the legacy flat
/// shape (`steps`), so seed files, imports and pre-migration rows keep loading.
/// Wiring semantics (orphaned ports, unreachable chains, gate policy) are checked
/// by the frontend validator, which is the one that also normalises documents.
pub fn validate_agent_spec(spec: &Value) -> Result<(), String> {
    let goal = spec
        .get("goal")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing required field 'goal'".to_string())?;
    if goal.trim().is_empty() {
        return Err("field 'goal' must not be empty".to_string());
    }

    match spec.get("chains").and_then(Value::as_array) {
        Some(chains) => validate_chains(chains),
        None => match spec.get("steps") {
            Some(_) => Ok(()),
            None => Err("missing required field 'chains'".to_string()),
        },
    }
}

fn validate_chains(chains: &[Value]) -> Result<(), String> {
    for (index, chain) in chains.iter().enumerate() {
        let label = format!("chains[{}]", index);
        if !chain.is_object() {
            return Err(format!("{} must be an object", label));
        }

        let capability = chain
            .get("capability")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if capability.is_empty() {
            return Err(format!("{} is missing 'capability'", label));
        }

        let result = chain
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if result.is_empty() {
            return Err(format!("{} is missing 'result'", label));
        }

        if let Some(gate) = chain.get("gate") {
            let mode = gate.get("mode").and_then(Value::as_str).unwrap_or("auto");
            if mode != "auto" && mode != "approve" {
                return Err(format!("{} has unsupported gate mode '{}'", label, mode));
            }
            if mode == "approve" {
                let approver = gate
                    .get("approver")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .unwrap_or("");
                if approver.is_empty() {
                    return Err(format!("{} needs an approver for its approval gate", label));
                }
            }
            if let Some(policy) = gate.get("onFailure").and_then(Value::as_str) {
                if !["stop", "retry", "escalate"].contains(&policy) {
                    return Err(format!("{} has unsupported failure policy '{}'", label, policy));
                }
            }
        }

        if let Some(inputs) = chain.get("inputs").and_then(Value::as_array) {
            for binding in inputs {
                let port = binding.get("port").and_then(Value::as_str).unwrap_or("");
                if port.trim().is_empty() {
                    return Err(format!("{} has an input without a port name", label));
                }
                match binding.get("from").and_then(Value::as_str).unwrap_or("parameter") {
                    "trigger" | "result" | "parameter" | "literal" => {}
                    other => {
                        return Err(format!(
                            "{} binds '{}' from unsupported source '{}'",
                            label, port, other
                        ))
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn chain_plan() -> Value {
        json!({
            "version": 2,
            "goal": "Answer customer questions",
            "triggers": [{"name": "customer_message", "kind": "event"}],
            "results": [
                {"name": "issue_summary", "terminal": false},
                {"name": "customer_response", "terminal": true}
            ],
            "chains": [
                {
                    "id": "cs-1",
                    "trigger": "customer_message",
                    "capability": "understand_issue",
                    "inputs": [{"port": "message", "from": "trigger", "ref": "customer_message"}],
                    "result": "issue_summary",
                    "gate": {"mode": "auto", "onFailure": "stop"}
                },
                {
                    "id": "cs-2",
                    "trigger": "issue_summary",
                    "capability": "draft_response",
                    "inputs": [{"port": "summary", "from": "result", "ref": "issue_summary"}],
                    "result": "customer_response",
                    "gate": {"mode": "approve", "approver": "support_lead", "onFailure": "escalate"}
                }
            ],
            "tools": ["understand_issue", "draft_response"],
            "knowledgeSources": []
        })
    }

    #[test]
    fn accepts_a_chain_plan() {
        assert!(validate_agent_spec(&chain_plan()).is_ok());
    }

    #[test]
    fn accepts_the_legacy_flat_plan() {
        let legacy = json!({"goal": "Do the thing", "steps": ["First", "Second"]});
        assert!(validate_agent_spec(&legacy).is_ok());
    }

    #[test]
    fn rejects_a_plan_without_a_goal() {
        let plan = json!({"goal": "  ", "chains": []});
        assert_eq!(
            validate_agent_spec(&plan).unwrap_err(),
            "field 'goal' must not be empty"
        );
    }

    #[test]
    fn rejects_an_approval_gate_without_an_approver() {
        let mut plan = chain_plan();
        plan["chains"][1]["gate"]["approver"] = json!("");
        let error = validate_agent_spec(&plan).unwrap_err();
        assert!(error.contains("needs an approver"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_an_unknown_binding_source() {
        let mut plan = chain_plan();
        plan["chains"][0]["inputs"][0]["from"] = json!("telepathy");
        let error = validate_agent_spec(&plan).unwrap_err();
        assert!(error.contains("unsupported source"), "unexpected error: {error}");
    }
}
