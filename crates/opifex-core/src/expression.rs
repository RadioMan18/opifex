//! Expression language parser for {{ ... }} references.
//! Pure Rust implementation — captures content between braces and parses it.

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Reference { path: Vec<String> },
    FunctionCall { name: String, args: Vec<Expr> },
    Literal(Literal),
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Eq, Neq, Lt, Gt, Le, Ge, And, Or,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("no {{...}} expression found in: {0}")]
    NoExpression(String),
    #[error("unsupported expression construct: {0}")]
    Unsupported(String),
    #[error("unbalanced braces in expression: {0}")]
    UnbalancedBraces(String),
}

/// Parse the first {{...}} expression found in the input string.
pub fn parse_expression(input: &str) -> Result<Expr, ParseError> {
    let start = input
        .find("{{")
        .ok_or_else(|| ParseError::NoExpression(input.to_string()))?;
    let rest = &input[start + 2..];
    let end = rest
        .find("}}")
        .ok_or_else(|| ParseError::UnbalancedBraces(input.to_string()))?;
    let content = &rest[..end];
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return Err(ParseError::Unsupported("empty expression content".to_string()));
    }

    parse_trimmed(trimmed)
}

/// Parse already-trimmed expression content (no {{ }}).
fn parse_trimmed(s: &str) -> Result<Expr, ParseError> {
    // Try function call: name(args)
    if let Some(paren_start) = s.find('(') {
        if let Some(paren_end) = s.rfind(')') {
            if paren_end > paren_start {
                let name = s[..paren_start].trim().to_string();
                let args_str = s[paren_start + 1..paren_end].trim();
                if args_str.is_empty() {
                    return Ok(Expr::FunctionCall { name, args: vec![] });
                }
                let args: Result<Vec<Expr>, _> = args_str
                    .split(',')
                    .map(|a| a.trim())
                    .filter(|a| !a.is_empty())
                    .map(|a| parse_trimmed(a))
                    .collect();
                return Ok(Expr::FunctionCall { name, args: args? });
            }
        }
    }

    // Try dotted reference: a.b.c
    if s.contains('.') {
        let path: Vec<String> = s.split('.').map(|p| p.trim().to_string()).collect();
        return Ok(Expr::Reference { path });
    }

    // Try literal
    if let Ok(num) = s.parse::<f64>() {
        return Ok(Expr::Literal(Literal::Number(num)));
    }
    match s {
        "true" => return Ok(Expr::Literal(Literal::Boolean(true))),
        "false" => return Ok(Expr::Literal(Literal::Boolean(false))),
        "null" => return Ok(Expr::Literal(Literal::Null)),
        _ => {}
    }

    // Plain identifier reference
    Ok(Expr::Reference { path: vec![s.to_string()] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_reference() {
        let result = parse_expression("{{ node }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Reference { path } => assert_eq!(path, vec!["node".to_string()]),
            _ => panic!("expected reference"),
        }
    }

    #[test]
    fn test_single_identifier() {
        let result = parse_expression("{{ input }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Reference { path } => assert_eq!(path, vec!["input".to_string()]),
            _ => panic!("expected reference"),
        }
    }

    #[test]
    fn test_number_literal() {
        let result = parse_expression("{{ 42 }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Literal(Literal::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("expected number literal"),
        }
    }

    #[test]
    fn test_boolean_literal() {
        let result = parse_expression("{{ true }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Literal(Literal::Boolean(b)) => assert_eq!(b, true),
            _ => panic!("expected boolean literal"),
        }
    }

    #[test]
    fn test_function_call_no_args() {
        let result = parse_expression("{{ foo() }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("expected function call"),
        }
    }

    #[test]
    fn test_function_call_with_arg() {
        let result = parse_expression("{{ upper(name) }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "upper");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("expected function call"),
        }
    }

    #[test]
    fn test_dotted_reference() {
        let result = parse_expression("{{ node.output }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Reference { path } => assert_eq!(path, vec!["node".to_string(), "output".to_string()]),
            _ => panic!("expected dotted reference"),
        }
    }

    #[test]
    fn test_null_literal() {
        let result = parse_expression("{{ null }}");
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Literal(Literal::Null) => {}
            _ => panic!("expected null literal"),
        }
    }

    #[test]
    fn test_no_braces_error() {
        let result = parse_expression("just text");
        assert!(result.is_err());
    }

    #[test]
    fn test_unbalanced_braces_error() {
        let result = parse_expression("{{ missing close");
        assert!(result.is_err());
    }
}
