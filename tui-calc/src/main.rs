mod calc;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() {
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
                println!("Interrupted (Ctrl+C)");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("EOF (Ctrl+D)");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
