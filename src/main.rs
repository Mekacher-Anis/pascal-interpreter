use std::env;
use std::fs;
use std::io;

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod symbols;
mod token;
mod visualizer;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use visualizer::Visualizer;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let content = fs::read_to_string(filename)?;

    let lexer = Lexer::new(&content);
    let mut parser = match Parser::new(lexer) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut visualizer = Visualizer::new();
    let svg_content = visualizer.generate_svg(&ast);
    if let Err(e) = std::fs::write("ast.svg", svg_content) {
        eprintln!("Error writing SVG: {}", e);
    } else {
        println!("AST visualization saved to ast.svg");
    }

    let mut interpreter = Interpreter::new();
    match interpreter.interpret(&ast) {
        Ok(_) => println!("program done"),
        Err(e) => eprintln!("Error: {}", e),
    }
    // Pretty print interpreter variables after execution completes
    interpreter.pretty_print_variables();

    println!("Symtable:\n{}", interpreter.symtab);

    Ok(())
}
