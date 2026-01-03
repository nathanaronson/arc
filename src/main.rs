use arc::{ArcError, VM};
use std::io::{self, Write};
use std::{env, fs, process};

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
    print_banner();
    let mut vm = VM::new();

    loop {
        print!(">>> ");
        io::stdout().flush().expect("Could not flush stdout");
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Could not read from stdin");
        if line.trim().is_empty() {
            break;
        }
        let _ = vm.interpret(&line);
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

fn print_banner() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let arch = env::consts::ARCH;
    let bits = usize::BITS as usize;
    let os = env::consts::OS;
    println!("{} {} ({}, {}-bit) on {}", name, version, arch, bits, os);
}
