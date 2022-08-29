use clap::Parser;
use run::run;

mod aggregate;
mod contracts;
mod error;
mod graph_client;
mod run;
mod settings;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Address to query
    #[clap(short, long)]
    user: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("You are querying positions of {}.", args.user);
    env_logger::init();

    if let Err(run_err) = run(&args.user).await {
        eprintln!("Failed to run settler: {:#?}", run_err);
        std::process::exit(1);
    }
}
