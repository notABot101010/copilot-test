mod calc;

use std::env;

use rustyline::{error::ReadlineError, DefaultEditor};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match calc::evaluate(args[1..].join(" ").as_str()) {
            Ok(result) => {
                println!("{}", result);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
        return;
    }

    println!("Terminal Calculator");
    println!("Supports: +, -, *, /, %, ^ (power), and parentheses");
    println!("Use arrow keys to navigate history. Press Ctrl+C or Ctrl+D to exit.\n");

    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Add to history only if it's a non-empty line
                let _ = rl.add_history_entry(line);

                match calc::evaluate(line) {
                    Ok(result) => {
                        println!("{}", result);
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
