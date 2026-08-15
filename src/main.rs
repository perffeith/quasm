use clap::{Args as ClapArgs, Parser as ClapParser, Subcommand};
use wazi::{lexer, parser, sema, lower, codegen};
use std::{fs, path::PathBuf};

#[derive(ClapParser)]
#[command(name = "wazi")]
struct Cli {
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand)]
enum Command {
    Build(BuildArgs),
    Run(BuildArgs)
}

#[derive(ClapArgs)]
struct BuildArgs {
    file: PathBuf,
    #[arg(short = 'd', long = "debug")]
    debug: bool
}

fn write_debug(name: &str, contents: &str) {
    let result = fs::create_dir_all("build/debug")
        .and_then(|_| fs::write(format!("build/debug/{name}"), contents));
    if let Err(e) = result {
        eprintln!("error writing build/debug/{name}: {e}");
    }
}

fn write_wasm(wasm: &[u8]) -> PathBuf {
    let path = PathBuf::from("build/out.wasm");
    fs::create_dir_all("build").and_then(|_| fs::write(&path, wasm))
        .unwrap_or_else(|e| {
            eprintln!("error writing wasm: {}", e);
            std::process::exit(1);
        });
    path
}

fn compile(args: &BuildArgs) -> Vec<u8> {
    let prelude = fs::read_to_string("builtin/prelude.wz")
        .expect("failed to read prelude");

    let src = fs::read_to_string(&args.file).unwrap_or_else(|e| {
        eprintln!("error reading '{}': {}", args.file.display(), e);
        std::process::exit(1);
    });

    let final_src = prelude + &src;

    let tokens = match lexer::lex(&final_src) {
        Ok(tokens) => {
            if args.debug { write_debug("tokens.txt", &lexer::debug_tokens(&tokens)); }
            tokens
        },
        Err(lexer_errors) => {
            for error in &lexer_errors {
                eprintln!("lex error: {} at {}", error.message, error.span);
            }
            std::process::exit(1);
        }
    };

    let ast = match parser::parse(tokens) {
        Ok(ast) => {
            if args.debug { write_debug("ast.txt", &format!("{:#?}", ast)); }
            ast
        },
        Err(e) => {
            eprintln!("parse error: {} at {}", e.message, e.span);
            std::process::exit(1);
        }
    };

    let tast = match sema::check(ast) {
        Ok(tast) => {
            if args.debug { write_debug("tast.txt", &format!("{:#?}", tast)); }
            tast
        },
        Err(e) => {
            eprintln!("sema error: {} at {}", e.message, e.span);
            std::process::exit(1);
        }
    };

    let ir = lower::lower(tast);
    if args.debug { write_debug("ir.txt", &format!("{:#?}", ir)); }

    codegen::emit(ir)
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Build(args) => {
            let wasm = compile(args);
            write_wasm(&wasm);
        },
        Command::Run(args) => {
            let _wasm = compile(args);
            todo!("maybe serve local server and run on the browser???");
        }
    }
}