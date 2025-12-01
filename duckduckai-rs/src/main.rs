use anyhow::Result;
use clap::Parser;
use duckduckai::DuckDuckGoClient;
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "duckduckai")]
#[command(about = "A simple CLI for DuckDuckGo AI chat", long_about = None)]
struct Args {
    /// The message to send to the AI
    #[arg(short, long)]
    message: Option<String>,

    /// The model to use (default: gpt-5-mini)
    #[arg(long, default_value = "gpt-5-mini")]
    model: String,

    /// Enable streaming output
    #[arg(short, long)]
    stream: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let filter = if args.debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let mut client = DuckDuckGoClient::new()?;

    if let Some(message) = args.message {
        // Single message mode
        if args.stream {
            client
                .chat_stream(&message, Some(&args.model), |chunk| {
                    print!("{}", chunk);
                    io::stdout().flush().unwrap();
                })
                .await?;
            println!(); // New line after streaming
        } else {
            let response = client.chat(&message, Some(&args.model)).await?;
            println!("{}", response);
        }
    } else {
        // Interactive mode
        println!("DuckDuckGo AI Chat (type 'exit' to quit)");
        println!("Model: {}", args.model);
        println!();

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if input == "exit" || input == "quit" {
                println!("Goodbye!");
                break;
            }

            if args.stream {
                client
                    .chat_stream(input, Some(&args.model), |chunk| {
                        print!("{}", chunk);
                        io::stdout().flush().unwrap();
                    })
                    .await?;
                println!("\n"); // Double new line for readability
            } else {
                match client.chat(input, Some(&args.model)).await {
                    Ok(response) => println!("{}\n", response),
                    Err(err) => eprintln!("Error: {}\n", err),
                }
            }
        }
    }

    Ok(())
}
