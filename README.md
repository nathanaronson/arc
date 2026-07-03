# Arc

## Description

Arc is a programming language written entirely in [Rust](https://www.rust-lang.org/). It consists of a token scanner, bytecode compiler, and stack-based virtual machine.

## Instructions

You will need [Cargo](https://doc.rust-lang.org/cargo/) installed to compile and run Arc.

### Run the Repl

```bash
cargo run
```

### Run a Specific File

```bash
cargo run <file_name.arc>
```

### Run the Web Playground

You will need [wasm-pack](https://drager.github.io/wasm-pack/) installed to build Arc for WebAssembly.

```bash
wasm-pack build --target web --out-dir web/pkg
python3 -m http.server 8000 --directory web
```

Then open <http://localhost:8000> in your browser.

## Live Playground

Currently hosted at my [personal website](https://seas.upenn.edu/~narons/arc/).
