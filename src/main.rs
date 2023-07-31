mod animal;
mod aquarium;
mod check;
mod data;
mod paths;
mod rules;
mod sexpr_format;
mod sexpr_impl;
mod tank;
mod util;

use check::*;
use clap::Parser;
use data::*;
use sexpr_format::*;
use std::error::Error;

fn main() {
    let opts = Opts::parse();
    let data = read_game_data().unwrap();

    match opts.command {
        SubCommand::Lookup(l) => {
            let mut did_write = false;

            for s in data.species {
                if s.id.contains(&l.search_term) {
                    did_write = true;
                    if l.debug {
                        println!("{:#?}", s);
                    } else {
                        println!("{}", PrettyPrinted { expr: s.to_sexp() });
                    }
                }
            }

            for t in data.tanks {
                if t.id.contains(&l.search_term) {
                    did_write = true;
                    if l.debug {
                        println!("{:#?}", t);
                    } else {
                        println!("{}", PrettyPrinted { expr: t.to_sexp() });
                    }
                }
            }

            if !did_write {
                println!("No entries found for search {}", l.search_term);
            }
        }

        SubCommand::Extract(e) => {
            let save = read_save(&data, &e.save_name).unwrap();

            if e.debug {
                //println!("{:#?}", save.to_spec());
            } else {
                //println!("{}", save.to_spec());
            }
        }

        SubCommand::Check(c) => {
            let args = CheckArgs {
                species: c.species,
                debug: c.debug,
                assume_all_fish_fully_grown: c.assume_fully_grown,
            };
            match check_for_viable_tank(&data, args) {
                Ok(_) => (),
                Err(error) => {
                    println!("{}", error);
                    std::process::exit(2);
                }
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
    Check(Check),
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

#[derive(Debug, Parser)]
struct Check {
    #[clap(value_parser = parse_key_val::<String,u16>)]
    species: Vec<(String, u16)>,
    /// Show debug-printed structs instead of pretty output
    #[clap(short)]
    debug: bool,
    /// Consider all fish fully grown for the purposes of predation
    #[clap(long)]
    assume_fully_grown: bool,
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
