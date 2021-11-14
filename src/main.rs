mod animal;
mod tank;
mod data;
mod paths;

use clap::{Parser};

fn main() {
    let opts = Opts::parse();

    match opts.command {
        SubCommand::Lookup(l) => {
            let data = data::read_game_data().unwrap();

            for s in data.species {
                if s.id.contains(&l.search_term) {
                    println!("{:?}", s);
                }
            }
        }
    }
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