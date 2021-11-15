mod animal;
mod aquarium;
mod data;
mod paths;
mod tank;

use clap::Parser;
use data::*;

fn main() {
    let opts = Opts::parse();
    let data = read_game_data().unwrap();

    match opts.command {
        SubCommand::Lookup(l) => {
            for s in data.species {
                if s.id.contains(&l.search_term) {
                    println!("{:#?}", s);
                }
            }
        }

        SubCommand::Extract(e) => {
            let save = read_save(&data, &e.save_name).unwrap();

            println!("{:#?}", save.to_spec());
        }
    }
}

#[derive(Parser)]
#[clap(version = "0.0", author = "Eric")]
struct Opts {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Lookup(Lookup),
    Extract(Extract),
}

#[derive(Debug, Parser)]
struct Lookup {
    search_term: String,
}

#[derive(Debug, Parser)]
struct Extract {
    save_name: String,
}
