use clap::{Parser};

fn main() {
    let opts = Opts::parse();

    match opts.command {
        SubCommand::Lookup(l) => {
            println!("Lookup {:?}!", l);
        }
    }

    println!("Hello, world!");
}

#[derive(Parser)]
#[clap(version = "0.0", author="Eric")]
struct Opts {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Lookup(Lookup),
}

#[derive(Debug)]
#[derive(Parser)]
struct Lookup {
    search_term: String,
}