mod animal;
mod aquarium;
mod data;
mod paths;
mod rules;
mod tank;

use animal::*;
use clap::Parser;
use data::*;

fn main() {
    let opts = Opts::parse();
    let data = read_game_data().unwrap();

    match opts.command {
        SubCommand::Lookup(l) => {
            for s in data.species {
                if s.id.contains(&l.search_term) {
                    if l.debug {
                        println!("{:#?}", s);
                    } else {
                        let a = Animal {
                            id: 0,
                            species: &s,
                            age: 0,
                        };
                        println!("{}", a.description());
                    }
                }
            }
            for t in data.tanks {
                if t.id.contains(&l.search_term) {
                    if l.debug {
                        println!("{:#?}", t);
                    } else {
                        println!("{}", t);
                    }
                }
            }
        }

        SubCommand::Extract(e) => {
            let save = read_save(&data, &e.save_name).unwrap();

            if e.debug {
                println!("{:#?}", save.to_spec());
            } else {
                println!("{}", save.to_spec());
            }
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
    /// Show debug-printed structs instead of pretty output
    #[clap(short)]
    debug: bool,
}

#[derive(Debug, Parser)]
struct Extract {
    save_name: String,
    /// Show debug-printed structs instead of pretty output
    #[clap(short)]
    debug: bool,
}
