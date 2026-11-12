use clap::Parser;
use opifex_core::can_connect;
use opifex_core::PortType;
use std::path::Path;

#[derive(Parser)]
#[command(name = "opifex-cli")]
#[command(about = "Opifex v2.0.0 headless CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Validate { graph: String },
    Export { graph: String, target: String },
    LintBlocks {
        #[arg(long, default_value = "docs/blocks")]
        specs_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Validate { graph } => {
            println!("Validate: {}", graph);
        }
        Commands::Export { graph, target } => {
            println!("Export {} to {}", graph, target);
        }
        Commands::LintBlocks { specs_dir } => {
            lint_blocks(&specs_dir);
        }
    }
}

fn lint_blocks(specs_dir: &str) {
    let mut errors = 0;

    // Rule 2a: Every registered block must have a markdown spec in docs/blocks/<block-id>.md
    let builtin_blocks = [
        "builtin:input",
        "builtin:llm-prompt",
        "builtin:structured-llm",
        "builtin:agent",
        "builtin:http-request",
        "builtin:condition",
        "builtin:loop-for-each",
        "builtin:merge",
        "builtin:code-js",
        "builtin:final-answer",
    ];

    println!("Linting block definitions against specs in {}", specs_dir);

    for block_id in &builtin_blocks {
        let spec_filename = block_id.replace(':', "_");
        let spec_path = Path::new(specs_dir).join(format!("{}.md", spec_filename));
        if spec_path.exists() {
            println!("  OK: {} has spec", block_id);
        } else {
            println!("  MISSING: {} -> {}", block_id, spec_path.display());
            errors += 1;
        }
    }

    // Rule 2b: Port conformance sanity check
    assert!(can_connect(&PortType::Any, &PortType::String));
    assert!(!can_connect(&PortType::String, &PortType::Number));
    println!("  OK: Port conformance checks pass");

    if errors > 0 {
        eprintln!("\nlint-blocks FAILED: {} missing block spec(s)", errors);
        std::process::exit(1);
    } else {
        println!("\nlint-blocks PASSED");
    }
}
