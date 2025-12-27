use arc::{ArcError, VM};
use std::io::Write;
use std::{env, fs, io, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Usage: {} [file]", args[0]);
            process::exit(64);
        }
    }
}

fn repl() {
    let mut vm = VM::new();

    loop {
        print!("> ");
        io::stdout().flush().expect("Could not flush stdout");
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Could not read from stdin");
        if line.trim().is_empty() {
            break;
        }
        vm.interpret(&line).ok();
    }
}

fn run_file(path: &str) {
    let mut vm = VM::new();
    let code = fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("Could not read file {}: {}", path, error);
        process::exit(74);
    });
    match vm.interpret(&code) {
        Ok(()) => {}
        Err(ArcError::CompileError) => process::exit(65),
        Err(ArcError::RuntimeError) => process::exit(70),
    }
}
