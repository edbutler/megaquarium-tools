// we have to stuff this all in here because rust formatting mangles the sexp! macro
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::rules::*;
use crate::tank::*;
use crate::animal::*;
use crate::sexpr_format::*;
use lexpr::*;

impl ToSexp for Species {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("species");

        builder.add("id", symbol_of_string(&self.id));
        if self.immobile {
            builder.add("immobile?", Value::Bool(true));
        }
        builder.add("size", self.size.final_size.into());
        if self.size.armored {
            builder.add("armored?", Value::Bool(true));
        }
        let constraints = lexpr::Value::list(self.constraints().iter().map(|c| c.to_sexp()));
        builder.add("constraints", constraints);

        builder.to_value()
    }
}

impl ToSexp for Constraint {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        match self {
            Constraint::Temperature(t) => {
                let e = symbol_of_str(match t {
                    Temperature::Cold => "cold",
                    Temperature::Warm => "warm"
                });
                sexp!((temperature ,e))
            }
            Constraint::Quality(q) => sexp!((quality ,(*q))),
            Constraint::NeedsFood {kind, daily_amount} =>
                sexp!((eats ,(symbol_of_string(kind)) ,(*daily_amount))),
            Constraint::Scavenger => sexp!((scavenger)),
            Constraint::Shoaler(s) => sexp!((shoaler ,(*s))),
            Constraint::IsBully => sexp!((bully)),
            Constraint::NoBully => sexp!((wimp)),
            Constraint::NoLight => sexp!((#"no-light")),
            Constraint::NeedsLight(l) => sexp!((light ,(*l))),
            Constraint::OnlyGenus(g) => sexp!((#"only-genus" ,(symbol_of_string(g)))),
            Constraint::NoGenus(g) => sexp!((#"no-genus" ,(symbol_of_string(g)))),
            Constraint::NoSpecies(s) => sexp!((#"no-species" ,(symbol_of_string(s)))),
            Constraint::NoFoodEaters(e) => sexp!((#"no-food-eaters" ,(symbol_of_string(e)))),
            Constraint::RoundedTank => sexp!((#"rounded-tank")),
            Constraint::TankSize(s) => sexp!((#"tank-size" ,(*s))),
            Constraint::Predator { kind, size } =>
                sexp!((predator ,(symbol_of_string(kind)) ,(*size))),
        }
    }
}

impl ToSexp for TankModel {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("tank-model");

        builder.add("id", symbol_of_string(&self.id));
        builder.add("min-size", Value::cons(self.min_size.0, self.min_size.1));
        builder.add("max-size", Value::cons(self.max_size.0, self.max_size.1));
        builder.add("density", self.density().into());
        if self.rounded {
            builder.add("rounded", Value::Bool(true));
        }

        builder.to_value()
    }
}

impl ToSexp for TankStatus {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("tank-status");

        builder.add("size", self.size.into());
        let temp = match self.environment.temperature {
            Temperature::Cold => "cold",
            Temperature::Warm => "warm"
        };
        builder.add("temperature", symbol_of_str(temp));
        let salinity = match self.environment.salinity {
            Salinity::Fresh => "fresh",
            Salinity::Salty => "salty"
        };
        builder.add("salinity", symbol_of_str(salinity));
        builder.add("quality", self.environment.quality.into());
        builder.add("lighting", self.lighting.into());
        if self.rounded {
            builder.add("rounded", Value::Bool(true));
        }

        builder.to_value()
    }
}
