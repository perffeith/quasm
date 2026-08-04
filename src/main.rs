use clap::Parser as ClapParser;
use quasm::{lexer, parser, sema, lower, codegen};
use std::{fs, path::PathBuf};

#[derive(ClapParser)]
struct Args {
    file: PathBuf,
    #[arg(short = 'd', long = "debug")]
    debug: bool
}

fn write_debug(name: &str, contents: &str) {
    let result = fs::create_dir_all("build").and_then(|_| fs::write(format!("build/{name}"), contents));
    if let Err(e) = result {
        eprintln!("error writing build/{name}: {e}");
    }
}

fn main() {
    let args = Args::parse();
    let src = fs::read_to_string(&args.file).unwrap_or_else(|e| {
        eprintln!("error reading '{}': {}", args.file.display(), e);
        std::process::exit(1);
    });

    let tokens = match lexer::lex(&src) {
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

    let wasm = codegen::emit(ir);
    fs::create_dir_all("build").and_then(|_| fs::write("build/out.wasm", wasm))
        .unwrap_or_else(|e| {
            eprintln!("error writing wasm: {}", e);
            std::process::exit(1);
        });
}
