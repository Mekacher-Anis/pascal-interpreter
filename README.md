# Simple Pascal Interpreter

A simple Pascal interpreter written in Rust. \
I mainly did this to improve my rust skills. It's by no mean a production ready thing, polished, maintanable, or good, it's just for me to learn. \
This project demonstrates the implementation of a basic compiler/interpreter pipeline including lexical analysis, parsing, semantic analysis, and execution.

## Features

*   **Lexer**: Tokenizes the input Pascal source code.
*   **Parser**: Constructs an Abstract Syntax Tree (AST) from the tokens.
*   **Semantic Analyzer**: Checks for semantic errors and builds symbol tables.
*   **Interpreter**: Executes the AST.
*   **Call Stack**: Manages function/procedure calls and scopes.
*   **Visualizer**: (Optional) Tools for visualizing the AST or execution state.

## Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
*   Cargo (included with Rust)

## Installation

1.  Clone the repository:
    ```bash
    git clone <repository-url>
    cd simple-pascal-interpreter
    ```

2.  Build the project:
    ```bash
    cargo build --release
    ```

## Usage

To run the interpreter, pass the path to a Pascal source file as an argument:

```bash
cargo run -- <filename.pas>
```

For example, to run the provided test file:

```bash
cargo run -- test.pas
```

## Example Code

The project includes a `test.pas` file with the following content:

```pascal
program Main;

procedure Alpha(a : integer; b : integer);
var x : integer;
begin
   x := (a + b ) * 2;
end;

begin { Main }

   Alpha(3 + 5, 7);  { procedure call }

end.  { Main }
```

## Project Structure

*   `src/main.rs`: Entry point of the application.
*   `src/lexer.rs`: Handles lexical analysis.
*   `src/parser.rs`: Handles parsing and AST construction.
*   `src/ast.rs`: Defines the Abstract Syntax Tree nodes.
*   `src/semantic_analyzer.rs`: Performs semantic checks.
*   `src/interpreter.rs`: Executes the program.
*   `src/symbols.rs`: Manages symbol tables.
*   `src/call_stack.rs`: Manages the runtime call stack.
*   `src/token.rs`: Defines token types.
*   `src/visualizer.rs`: Utilities for visualization.

## Testing

You can test the interpreter by running it against the provided `test.pas` file or by creating your own Pascal files.

```bash
cargo run -- test.pas
```
