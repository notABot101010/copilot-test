use anyhow::Result;
use clap::{Parser, Subcommand};
use duckduckai::{DuckDuckGoClient, run_server, DEFAULT_MODEL};
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "duckduckai")]
#[command(about = "A simple CLI for DuckDuckGo AI chat", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The message to send to the AI (only for chat mode)
    #[arg(short, long)]
    message: Option<String>,

    /// The model to use (default: gpt5-mini)
    #[arg(long, default_value = DEFAULT_MODEL)]
    model: String,

    /// Enable streaming output
    #[arg(short, long)]
    stream: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start an OpenAI-compatible API server
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to bind to
        #[arg(long, default_value = "3000")]
        port: u16,

        /// API key for authentication
        #[arg(long)]
        api_key: String,

        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle serve subcommand
    if let Some(Commands::Serve {
        host,
        port,
        api_key,
        debug,
    }) = args.command
    {
        // Setup logging for server
        let filter = if debug {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new("info")
        };

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .init();

        return run_server(&host, port, api_key).await;
    }

    // Setup logging for chat mode
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
