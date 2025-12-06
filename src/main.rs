use std::io;

pub mod calc;

fn main() -> io::Result<()> {
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        if buffer.len() == 0 {
            continue;
        }
        if buffer == "/q" {
            break;
        }

        let mut interpreter = calc::Interpreter::new(&buffer);
        let result = interpreter.expr();
        dbg!(result);
    }
    std::io::Result::Ok(())
}
