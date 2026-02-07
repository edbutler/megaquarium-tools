// pattern: Imperative Shell

use std::fmt::Display;

use crate::check::{AquariumCheckResult, CheckQuery, ExhibitCheckResult, ExhibitValidation};
use crate::rules::Violation;
use crate::sexpr_format::PrettyPrinted;
use crate::sexpr_format::ToSexp;
use crate::tank::Environment;

pub fn print_violations(violations: &[Violation]) {
    let mut messages: Vec<_> = violations.iter().map(|v| v.to_string()).collect();
    messages.sort();
    messages.dedup();

    for v in messages {
        println!("- {}", v);
    }
}

pub fn print_exhibit_result(args: &CheckQuery, result: &ExhibitCheckResult) {
    println!("For contents:");
    for c in &args.counts {
        println!("- {}x {}", c.count, c.species);
    }

    if result.is_okay() {
        println!("\nThe minimum viable tank is:");
        if args.debug {
            println!("{:#?}", result.minimum_viable_environment);
        } else {
            println!(
                "{}",
                PrettyPrinted {
                    expr: result.minimum_viable_environment.to_sexp()
                }
            );
        }

        println!("\nWill require food (average per day):");
        for item in &result.food {
            println!("- {}x {}", item.count, item.food);
        }
    } else {
        println!("\nA valid tank is not possible:");
        print_violations(&result.violations);
    }
}

pub fn print_environment_differences(old: &Environment, new: &Environment) {
    fn format_opt<T>(x: Option<T>) -> String
    where
        T: Display,
    {
        match x {
            Some(v) => format!("{}", v),
            None => "n/a".to_string(),
        }
    }

    fn compare<T>(name: &str, old: T, new: T)
    where
        T: Display + PartialOrd,
    {
        if old < new {
            println!("- {}: {} → {}", name, old, new);
        }
    }

    fn compare_opt<T>(name: &str, old: Option<T>, new: Option<T>)
    where
        T: Display + PartialOrd,
    {
        if old < new {
            println!("- {}: {} → {}", name, format_opt(old), format_opt(new));
        }
    }

    compare("size", old.size, new.size);
    compare("quality", old.quality, new.quality);
    compare_opt("plants", old.plants, new.plants);
    compare_opt("rocks", old.rocks, new.rocks);
    compare_opt("caves", old.caves, new.caves);
    compare_opt("light", old.light, new.light);
}

fn print_exhibit_environment(exhibit: &ExhibitValidation) {
    let loaded = &exhibit.loaded_environment;
    let needed = &exhibit.minimum_viable_environment;

    // size is {needed} / {actual} because it's an upper bound
    println!("- size: {}/{}", needed.size, loaded.size);
    println!("- quality: {}%", needed.quality);

    // the rest are {actual} / {needed} because they're lower bounds
    if let Some(v) = needed.light {
        println!("- light: {}/{}", loaded.light, v);
    }
    if let Some(v) = needed.plants {
        println!("- plants: {}/{}", loaded.plants, v);
    }
    if let Some(v) = needed.rocks {
        println!("- rocks: {}/{}", loaded.rocks, v);
    }
    if let Some(v) = needed.caves {
        println!("- caves: {}/{}", loaded.caves, v);
    }
    if let Some(v) = needed.bogwood {
        println!("- bogwood: {}/{}", loaded.bogwood, v);
    }
    if let Some(v) = needed.flat_surfaces {
        println!("- flat_surfaces: {}/{}", loaded.flat_surfaces, v);
    }
    if let Some(v) = needed.vertical_surfaces {
        println!("- vertical_surfaces: {}/{}", loaded.vertical_surfaces, v);
    }
    if let Some(v) = needed.fluffy_foliage {
        println!("- fluffy_foliage: {}/{}", loaded.fluffy_foliage, v);
    }
    if let Some(v) = needed.different_decorations {
        println!("- different_decorations: {}/{}", loaded.different_decorations, v);
    }
    if let Some(v) = needed.interior {
        println!("- interior: {}/{}", loaded.interior.map_or("none".to_string(), |i| i.to_string()), v);
    }
}

pub fn print_aquarium_result(result: &AquariumCheckResult, debug: bool) {
    println!("Checking {} tanks...", result.exhibits.len());

    for exhibit in &result.exhibits {
        println!("{}:", exhibit.name);

        if debug {
            println!("loaded: {:#?}", exhibit.loaded_environment);
            println!("needed: {:#?}", exhibit.minimum_viable_environment);
        } else {
            print_exhibit_environment(exhibit);
        }

        for item in &exhibit.food {
            println!("- {}x {}", item.count, item.food);
        }

        print_violations(&exhibit.violations);
    }

    if result.is_okay() {
        println!("No problems!");
    }
}
