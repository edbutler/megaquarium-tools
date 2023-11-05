#![feature(trait_alias)]

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

use aquarium::*;
use check::*;
use clap::{Parser, ValueEnum};
use data::*;
use sexpr_format::*;
use std::error::Error;

use crate::rules::{ExhibitSpec, RuleOptions};

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
            let desc = save.description(e.summary);

            if e.debug {
                println!("{:#?}", desc);
            } else {
                println!("{}", PrettyPrinted { expr: desc.to_sexp() });
            }
        }

        SubCommand::Check(c) => {
            fn do_work(c: Check, data: &GameData) -> util::Result<()> {
                let counts = make_species_counts(c.species);
                let args = CheckArgs {
                    species: &counts,
                    debug: c.debug,
                    assume_all_fish_fully_grown: c.assume_fully_grown,
                };
                let animals = animals_from_counts(data, &args)?;
                let result = check_for_viable_tank(&data, &animals);
                print_check_result(&args, &result);
                Ok(())
            }

            match do_work(c, &data) {
                Ok(_) => (),
                Err(error) => {
                    println!("{}", error);
                    std::process::exit(2);
                }
            }
        }

        SubCommand::List(list) => match list.kind {
            ListOptions::Animals => {
                println!("Animals:");
                for x in data.species {
                    println!("- {}", x.id);
                }
            }
            ListOptions::Tanks => {
                println!("Tanks:");
                for x in data.tanks {
                    println!("- {}", x.id);
                }
            }
            ListOptions::Food => {
                println!("Food:");
                for x in data.food {
                    println!("- {}", x);
                }
            }
        },

        SubCommand::Validate(_) => {
            fn do_work(data: &GameData) -> util::Result<()> {
                let options = RuleOptions {
                    assume_all_fish_fully_grown: false,
                };
                let aquarium = load_aquarium_from_stdin()?.to_ref(data, &options)?;
                let args = ValidateArgs {
                    aquarium: &aquarium,
                    debug: false,
                    assume_all_fish_fully_grown: false,
                };
                check_for_viable_aquarium(data, &args)?;
                Ok(())
            }

            match do_work(&data) {
                Ok(_) => (),
                Err(error) => {
                    println!("{}", error);
                    std::process::exit(2);
                }
            }
        }

        SubCommand::Expand(e) => {
            fn do_work(e: Expand, data: &GameData) -> util::Result<()> {
                let options = RuleOptions {
                    assume_all_fish_fully_grown: false,
                };

                let aquarium = load_aquarium_from_stdin()?.to_ref(data, &options)?;

                let counts = make_species_counts(e.species);

                let args = CheckArgs {
                    species: &counts,
                    debug: false,
                    assume_all_fish_fully_grown: false,
                };

                let new_animals = animals_from_counts(data, &args)?;

                let base_result = check_for_viable_tank(&data, &new_animals);

                if !base_result.is_okay() {
                    print_check_result(&args, &base_result);
                    return Ok(());
                }

                let expansion = ExhibitSpec {
                    animals: &new_animals,
                    environment: base_result.minimum_viable_environment,
                };

                println!("New fish will use {} additional tank size", expansion.environment.size);

                let mut can_add_somewhere = false;

                for exhibit in &aquarium.exhibits {
                    let expand_result = try_expand_tank(data, exhibit, &expansion);

                    let is_okay = expand_result.is_okay();
                    let do_print = e.all || is_okay;

                    if do_print {
                        println!("{} add to {}", if is_okay { "Can" } else { "Cannot" }, exhibit.name);
                    }

                    if is_okay {
                        can_add_somewhere = true;
                    }

                    if do_print && exhibit.animals.len() > 0 {
                        print_violations(&expand_result.violations);
                        let original_environment = environment_for_exhibit(exhibit);
                        print_environment_differences(&original_environment, &expand_result.minimum_viable_environment);
                    }
                }

                if !can_add_somewhere {
                    println!("Unable to add to current aquarium!");
                }

                Ok(())
            }

            match do_work(e, &data) {
                Ok(_) => (),
                Err(error) => {
                    println!("{}", error);
                    std::process::exit(2);
                }
            }
        }
    }
}

fn make_species_counts(counts: Vec<(String, u16)>) -> Vec<SpeciesCount> {
    counts.into_iter().map(|(species, count)| SpeciesCount { species, count }).collect()
}

fn load_aquarium_from_stdin() -> util::Result<AquariumDesc> {
    let stdin = std::io::stdin();
    from_reader::<std::io::Stdin, AquariumDesc>(stdin)
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
    List(List),
    Validate(Validate),
    Expand(Expand),
}

/// Show information about the any game entity for a given search string.
#[derive(Debug, Parser)]
struct Lookup {
    search_term: String,
    /// Show debug-printed structs instead of pretty output
    #[clap(short)]
    debug: bool,
}

/// Print an aquarium summary in s-expression format to stdout for a given save filename
#[derive(Debug, Parser)]
struct Extract {
    save_name: String,
    /// Extract a summary of animals instead of individuals, will not have age.
    #[clap(short)]
    summary: bool,
    /// Show debug-printed structs instead of pretty output
    #[clap(short)]
    debug: bool,
}

/// Check the validity of the given set of animals, printing the minimum viable tank
#[derive(Debug, Parser)]
struct Check {
    /// A set of species/count pairs, e.g., `clown_fish=3 anemone=2`
    #[clap(value_parser = parse_key_val::<String,u16>)]
    species: Vec<(String, u16)>,
    /// Show debug-printed structs instead of pretty output
    #[clap(short)]
    debug: bool,
    /// Consider all fish fully grown for the purposes of predation
    #[clap(long)]
    assume_fully_grown: bool,
}

/// Validates an aquarium provided over stdin
#[derive(Debug, Parser)]
struct Validate {}

/// List which tanks could support the addition of the given fish
#[derive(Debug, Parser)]
struct Expand {
    /// A set of species/count pairs, e.g., `clown_fish=3 anemone=2`
    #[clap(value_parser = parse_key_val::<String,u16>)]
    species: Vec<(String, u16)>,
    /// Show all tanks (with potential violations) even if they cannot support the additions.
    /// Default is to only show valid tanks
    #[clap(short)]
    all: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum ListOptions {
    Animals,
    Tanks,
    Food,
}

/// List various game objects
#[derive(Debug, Parser)]
struct List {
    kind: ListOptions,
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
