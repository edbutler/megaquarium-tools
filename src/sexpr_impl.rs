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

        builder.add("size", sexp!(,(self.size.final_size)));

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
