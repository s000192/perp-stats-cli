use clap::Parser;
use run::run;

mod contracts;
mod error;
mod graph_client;
mod run;
mod settings;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    name: String,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

#[tokio::main]
async fn main() {
    // let args = Args::parse();

    // for _ in 0..args.count {
    //     println!("Hello {}!", args.name)
    // }
    env_logger::init();

    if let Err(run_err) = run().await {
        eprintln!("Failed to run settler: {:#?}", run_err);
        std::process::exit(1);
    }
}