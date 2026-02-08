//! Redox CLI - Command line interface for the Rust to Iron transpiler

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "redox")]
#[command(about = "A Rust to Iron transpiler")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transpile Rust source to Iron
    Reduce {
        /// Input Rust source file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output Iron file (default: stdout)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Validate output contains no prohibited symbols
        #[arg(short, long)]
        validate: bool,

        /// Show verbose error messages
        #[arg(short = 'V', long)]
        verbose: bool,
    },

    /// Validate Iron code
    Validate {
        /// Input Iron file to validate
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },

    /// Transpile Iron source to Rust
    Oxidize {
        /// Input Iron source file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output Rust file (default: stdout)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Show verbose error messages
        #[arg(short = 'V', long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Reduce {
            input,
            output,
            validate,
            verbose,
        } => {
            if let Err(e) = transpile_file(input, output, validate, verbose) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Commands::Validate { input } => {
            if let Err(e) = validate_file(input) {
                eprintln!("Validation error: {}", e);
                process::exit(1);
            }
        }
        Commands::Oxidize {
            input,
            output,
            verbose,
        } => {
            if let Err(e) = oxidize_file(input, output, verbose) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}

fn transpile_file(
    input: PathBuf,
    output: Option<PathBuf>,
    validate: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read input file
    let source = fs::read_to_string(&input)
        .map_err(|e| format!("Failed to read input file '{}': {}", input.display(), e))?;

    if verbose {
        eprintln!("Transpiling: {}", input.display());
        eprintln!("Source size: {} bytes", source.len());
    }

    // Transpile
    let iron_code = match redox::transpile(&source) {
        Ok(code) => code,
        Err(e) => {
            return Err(format!("Transpilation failed: {}", e).into());
        }
    };

    if verbose {
        eprintln!("Output size: {} bytes", iron_code.len());
    }

    // Validate if requested
    if validate {
        if !redox::validate_iron(&iron_code) {
            eprintln!("Warning: Output contains prohibited symbols!");
            eprintln!("This indicates a bug in the transpiler.");
        } else if verbose {
            eprintln!("Validation passed: No prohibited symbols found");
        }
    }

    // Output result
    match output {
        Some(path) => {
            fs::write(&path, iron_code)
                .map_err(|e| format!("Failed to write output file '{}': {}", path.display(), e))?;
            if verbose {
                eprintln!("Output written to: {}", path.display());
            }
        }
        None => {
            print!("{}", iron_code);
        }
    }

    Ok(())
}

fn validate_file(input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&input)
        .map_err(|e| format!("Failed to read file '{}': {}", input.display(), e))?;

    if redox::validate_iron(&content) {
        println!("✓ Valid Iron code");
        Ok(())
    } else {
        Err("Invalid Iron code: contains prohibited symbols".into())
    }
}

fn oxidize_file(
    input: PathBuf,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read input file
    let source = fs::read_to_string(&input)
        .map_err(|e| format!("Failed to read input file '{}': {}", input.display(), e))?;

    if verbose {
        eprintln!("Oxidizing: {}", input.display());
        eprintln!("Source size: {} bytes", source.len());
    }

    // Oxidize
    let rust_code = match redox::oxidize(&source) {
        Ok(code) => code,
        Err(e) => {
            return Err(format!("Oxidation failed: {}", e).into());
        }
    };

    if verbose {
        eprintln!("Output size: {} bytes", rust_code.len());
    }

    // Output result
    match output {
        Some(path) => {
            fs::write(&path, rust_code)
                .map_err(|e| format!("Failed to write output file '{}': {}", path.display(), e))?;
            if verbose {
                eprintln!("Output written to: {}", path.display());
            }
        }
        None => {
            print!("{}", rust_code);
        }
    }

    Ok(())
}
