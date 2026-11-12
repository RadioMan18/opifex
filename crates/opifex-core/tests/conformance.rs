use opifex_core::validator::can_connect;
use opifex_core::PortType;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct PortConformanceVector {
    source_type: String,
    target_type: String,
    expected_result: bool,
}

fn parse_port_type(s: &str) -> PortType {
    match s {
        "string" => PortType::String,
        "number" => PortType::Number,
        "boolean" => PortType::Boolean,
        "object" => PortType::Object,
        "any" => PortType::Any,
        "message" => PortType::Message,
        "llm" => PortType::LLM,
        "tool" => PortType::Tool,
        "json" => PortType::Json,
        _ => panic!("unknown port type: {}", s),
    }
}

#[test]
fn test_type_conformance_vectors() {
    let vectors_json =
        std::fs::read_to_string("tests/vectors/port_conformance.json")
            .expect("Failed to load shared conformance vectors");

    let vectors: Vec<PortConformanceVector> =
        serde_json::from_str(&vectors_json).expect("Failed to parse conformance vectors");

    for vec in vectors {
        let source = parse_port_type(&vec.source_type);
        let target = parse_port_type(&vec.target_type);
        let result = can_connect(&source, &target);
        assert_eq!(
            result, vec.expected_result,
            "Conformance failure on vector: {} -> {}",
            vec.source_type, vec.target_type
        );
    }
}
