mod animal;
mod data;
mod paths;
mod tank;
mod aquarium;

use clap::Parser;

fn main() {
    let opts = Opts::parse();
    let data = data::read_game_data().unwrap();

    match opts.command {
        SubCommand::Lookup(l) => {
            for s in data.species {
                if s.id.contains(&l.search_term) {
                    println!("{:#?}", s);
                }
            }
        },

        SubCommand::Extract(e) => {

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
