//! LangGraph exporter for Opifex v2.0.0.
//! Generates a runnable Python project targeting the langgraph library.

use serde_json::json;
use std::collections::HashMap;

/// Capability report from an exporter check.
#[derive(Debug, Default, Clone)]
pub struct CapabilityReport {
    pub unsupported: Vec<(String, String)>, // (node_id, reason)
    pub warnings: Vec<String>,
}

impl CapabilityReport {
    pub fn add_unsupported(&mut self, node_id: String, reason: String) {
        self.unsupported.push((node_id, reason));
    }

    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    pub fn is_clean(&self) -> bool {
        self.unsupported.is_empty()
    }
}

/// Export options.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub include_readme: bool,
    pub pinned_dependencies: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_readme: true,
            pinned_dependencies: true,
        }
    }
}

/// An export artifact (file path + content).
#[derive(Debug, Clone)]
pub struct ExportArtifact {
    pub relative_path: String,
    pub content: String,
}

/// Exporter trait — each target language implements this.
pub trait Exporter {
    fn id(&self) -> &'static str;
    fn check(&self, graph: &ResolvedGraph) -> CapabilityReport;
    fn export(&self, graph: &ResolvedGraph, opts: &ExportOptions) -> Result<Vec<ExportArtifact>, ExportError>;
}

/// A resolved graph ready for export.
#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    pub id: String,
    pub name: String,
    pub nodes: Vec<ResolvedNode>,
    pub edges: Vec<ResolvedEdge>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedNode {
    pub id: String,
    pub block_id: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("export failed: {0}")]
    Failed(String),
}

/// LangGraph (Python) exporter.
pub struct LangGraphExporter;

impl Exporter for LangGraphExporter {
    fn id(&self) -> &'static str {
        "langgraph"
    }

    fn check(&self, graph: &ResolvedGraph) -> CapabilityReport {
        let mut report = CapabilityReport::default();
        for node in &graph.nodes {
            if node.block_id == "builtin:code-js" {
                report.add_unsupported(
                    node.id.clone(),
                    "JS Code block non-portable to Python target.".to_string(),
                );
            }
        }
        report
    }

    fn export(
        &self,
        graph: &ResolvedGraph,
        opts: &ExportOptions,
    ) -> Result<Vec<ExportArtifact>, ExportError> {
        let mut artifacts = Vec::new();

        // Generate Python graph code
        let graph_code = self.generate_python_code(graph);
        artifacts.push(ExportArtifact {
            relative_path: "graph.py".to_string(),
            content: graph_code,
        });

        // Generate requirements.txt with pinned versions
        let requirements = if opts.pinned_dependencies {
            "langgraph==0.2.20\nlangchain-core==0.3.0\npydantic==2.8.0\n".to_string()
        } else {
            "langgraph\nlangchain-core\npydantic\n".to_string()
        };
        artifacts.push(ExportArtifact {
            relative_path: "requirements.txt".to_string(),
            content: requirements,
        });

        // Generate README.md
        if opts.include_readme {
            let readme = format!(
                "# Generated Agent: {}\n\n## Export Target: LangGraph\n\nRun:\n```bash\npip install -r requirements.txt\npython graph.py\n```\n\n## Nodes\n{}\n\n## Edges\n{}\n",
                graph.name,
                graph.nodes.iter().map(|n| format!("- {} ({})", n.id, n.block_id)).collect::<Vec<_>>().join("\n"),
                graph.edges.iter().map(|e| format!("- {} -> {}", e.source, e.target)).collect::<Vec<_>>().join("\n"),
            );
            artifacts.push(ExportArtifact {
                relative_path: "README.md".to_string(),
                content: readme,
            });
        }

        Ok(artifacts)
    }
}

impl LangGraphExporter {
    fn generate_python_code(&self, graph: &ResolvedGraph) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("from typing import Any, Dict, List, Optional".to_string());
        lines.push("from langgraph.graph import StateGraph, END".to_string());
        lines.push("import json".to_string());
        lines.push("".to_string());
        lines.push(format!("class AgentState:"));
        lines.push("    \"\"\"Shared state passed between nodes.\"\"\"".to_string());
        lines.push("    messages: List[Dict[str, Any]]".to_string());
        lines.push("    current_output: Optional[Any]".to_string());
        lines.push("".to_string());

        // Generate node functions
        for node in &graph.nodes {
            lines.push(format!("def {}_node(state: AgentState) -> AgentState:", node.id));
            lines.push(format!("    \"\"\"Executes block: {}\"\"\"", node.block_id));
            lines.push("    # TODO: Implement block logic".to_string());
            lines.push("    return state".to_string());
            lines.push("".to_string());
        }

        // Build graph
        lines.push("def build_graph():".to_string());
        lines.push("    workflow = StateGraph(AgentState)".to_string());
        for node in &graph.nodes {
            lines.push(format!("    workflow.add_node(\"{}\", {}_node)", node.id, node.id));
        }
        // Set entry point (first trigger node)
        if let Some(first) = graph.nodes.first() {
            lines.push(format!("    workflow.set_entry_point(\"{}\")", first.id));
        }
        // Add edges
        for edge in &graph.edges {
            lines.push(format!("    workflow.add_edge(\"{}\", \"{}\")", edge.source, edge.target));
        }
        // Add final edge to END
        if let Some(last) = graph.nodes.last() {
            lines.push(format!("    workflow.add_edge(\"{}\", END)", last.id));
        }
        lines.push("    return workflow.compile()".to_string());
        lines.push("".to_string());
        lines.push("if __name__ == \"__main__\":".to_string());
        lines.push("    app = build_graph()".to_string());
        lines.push("    result = app.invoke({\"messages\": [], \"current_output\": None})".to_string());
        lines.push("    print(result)".to_string());

        lines.join("\n")
    }
}

/// OpenAI Agents SDK exporter.
pub struct OpenAIAgentsExporter;

impl Exporter for OpenAIAgentsExporter {
    fn id(&self) -> &'static str {
        "openai-agents"
    }

    fn check(&self, graph: &ResolvedGraph) -> CapabilityReport {
        let mut report = CapabilityReport::default();
        for node in &graph.nodes {
            if node.block_id == "builtin:code-js" {
                report.add_unsupported(
                    node.id.clone(),
                    "JS Code block non-portable to Python target.".to_string(),
                );
            }
        }
        report
    }

    fn export(
        &self,
        graph: &ResolvedGraph,
        opts: &ExportOptions,
    ) -> Result<Vec<ExportArtifact>, ExportError> {
        let mut artifacts = Vec::new();

        let agent_code = self.generate_python_code(graph);
        artifacts.push(ExportArtifact {
            relative_path: "agent.py".to_string(),
            content: agent_code,
        });

        let requirements = if opts.pinned_dependencies {
            "openai-agents==0.1.0\npydantic==2.8.0\n".to_string()
        } else {
            "openai-agents\npydantic\n".to_string()
        };
        artifacts.push(ExportArtifact {
            relative_path: "requirements.txt".to_string(),
            content: requirements,
        });

        if opts.include_readme {
            let readme = format!(
                "# Generated Agent: {}\n\n## Export Target: OpenAI Agents SDK\n\nRun:\n```bash\npip install -r requirements.txt\npython agent.py\n```\n",
                graph.name
            );
            artifacts.push(ExportArtifact {
                relative_path: "README.md".to_string(),
                content: readme,
            });
        }

        Ok(artifacts)
    }
}

impl OpenAIAgentsExporter {
    fn generate_python_code(&self, graph: &ResolvedGraph) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("from agents import Agent, Runner".to_string());
        lines.push("import asyncio".to_string());
        lines.push("".to_string());
        lines.push(format!("agent = Agent("));
        lines.push(format!("    name=\"{}\",", graph.name));
        lines.push("    instructions=\"Generated by Opifex v2.0.0\",".to_string());
        lines.push(")".to_string());
        lines.push("".to_string());
        lines.push("async def main():".to_string());
        lines.push("    result = await Runner.run(agent, \"Hello\")".to_string());
        lines.push("    print(result.final_output)".to_string());
        lines.push("".to_string());
        lines.push("if __name__ == \"__main__\":".to_string());
        lines.push("    asyncio.run(main())".to_string());

        lines.join("\n")
    }
}

/// Canonical JSON exporter.
pub struct JsonExporter;

impl Exporter for JsonExporter {
    fn id(&self) -> &'static str {
        "json"
    }

    fn check(&self, _graph: &ResolvedGraph) -> CapabilityReport {
        CapabilityReport::default()
    }

    fn export(
        &self,
        graph: &ResolvedGraph,
        opts: &ExportOptions,
    ) -> Result<Vec<ExportArtifact>, ExportError> {
        let mut artifacts = Vec::new();

        let json = json!({
            "version": "opifex.agent/1",
            "name": graph.name,
            "nodes": graph.nodes,
            "edges": graph.edges,
        });

        artifacts.push(ExportArtifact {
            relative_path: "agent.json".to_string(),
            content: serde_json::to_string_pretty(&json).map_err(|e| ExportError::Failed(e.to_string()))?,
        });

        if opts.include_readme {
            let readme = format!(
                "# Opifex Agent: {}\n\nCanonical JSON export. Run with `opifex-cli run`.\n",
                graph.name
            );
            artifacts.push(ExportArtifact {
                relative_path: "README.md".to_string(),
                content: readme,
            });
        }

        Ok(artifacts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_graph() -> ResolvedGraph {
        ResolvedGraph {
            id: "test-graph".to_string(),
            name: "Test Agent".to_string(),
            nodes: vec![
                ResolvedNode {
                    id: "n1".to_string(),
                    block_id: "builtin:input".to_string(),
                    parameters: HashMap::new(),
                },
                ResolvedNode {
                    id: "n2".to_string(),
                    block_id: "builtin:llm-prompt".to_string(),
                    parameters: HashMap::new(),
                },
                ResolvedNode {
                    id: "n3".to_string(),
                    block_id: "builtin:final-answer".to_string(),
                    parameters: HashMap::new(),
                },
            ],
            edges: vec![
                ResolvedEdge {
                    source: "n1".to_string(),
                    target: "n2".to_string(),
                },
                ResolvedEdge {
                    source: "n2".to_string(),
                    target: "n3".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_langgraph_export() {
        let exporter = LangGraphExporter;
        let graph = test_graph();
        let report = exporter.check(&graph);
        assert!(report.is_clean());

        let artifacts = exporter.export(&graph, &ExportOptions::default()).unwrap();
        let paths: Vec<&str> = artifacts.iter().map(|a| a.relative_path.as_str()).collect();
        assert!(paths.contains(&"graph.py"));
        assert!(paths.contains(&"requirements.txt"));
        assert!(paths.contains(&"README.md"));

        // Verify generated Python is syntactically reasonable
        let py_content = artifacts.iter().find(|a| a.relative_path == "graph.py").unwrap().content.clone();
        assert!(py_content.contains("from langgraph.graph import StateGraph"));
        assert!(py_content.contains("def build_graph()"));
    }

    #[test]
    fn test_openai_agents_export() {
        let exporter = OpenAIAgentsExporter;
        let graph = test_graph();
        let artifacts = exporter.export(&graph, &ExportOptions::default()).unwrap();
        let paths: Vec<&str> = artifacts.iter().map(|a| a.relative_path.as_str()).collect();
        assert!(paths.contains(&"agent.py"));
        assert!(paths.contains(&"requirements.txt"));
    }

    #[test]
    fn test_json_export() {
        let exporter = JsonExporter;
        let graph = test_graph();
        let artifacts = exporter.export(&graph, &ExportOptions::default()).unwrap();
        let paths: Vec<&str> = artifacts.iter().map(|a| a.relative_path.as_str()).collect();
        assert!(paths.contains(&"agent.json"));

        let json_content = artifacts.iter().find(|a| a.relative_path == "agent.json").unwrap().content.clone();
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        assert_eq!(parsed["version"], "opifex.agent/1");
        assert_eq!(parsed["name"], "Test Agent");
        assert_eq!(parsed["nodes"].as_array().unwrap().len(), 3);
        assert_eq!(parsed["edges"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_code_js_unsupported_in_python() {
        let exporter = LangGraphExporter;
        let mut graph = test_graph();
        graph.nodes.push(ResolvedNode {
            id: "n4".to_string(),
            block_id: "builtin:code-js".to_string(),
            parameters: HashMap::new(),
        });
        let report = exporter.check(&graph);
        assert!(!report.is_clean());
        assert_eq!(report.unsupported.len(), 1);
        assert_eq!(report.unsupported[0].0, "n4");
    }
}
