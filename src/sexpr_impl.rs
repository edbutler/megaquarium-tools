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
        builder.add("kind", symbol_of_string(&self.kind));

        let size = if self.immobile {
            Value::symbol("immobile")
        } else {
            Value::Number(self.maximum_used_tank_capacity().into())
        };
        builder.add("size", size);

        builder.add("environment", self.environment.to_sexp());

        if self.size.armored {
            builder.add("armored?", Value::Bool(true));
        }

        if let Some(f) = &self.fighting {
            builder.add("fighting", invoke_symbol(f.as_str()));
        }

        //constraints.extend(self.constraints().iter().map(|c| c.to_sexp()));

        //builder.add("constraints", lexpr::Value::list(constraints));

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

fn add_if_positive(builder: &mut StructBuilder, key: &str, x:u16) {
    if x > 0 {
        builder.add(key, x.into())
    }
}

impl Environment {
    #[allow(unused_parens)]
    fn add_to_sexp(&self, builder: &mut StructBuilder) {
        builder.add("temperature", symbol_of_str(self.temperature.as_str()));
        let salinity = match self.salinity {
            Salinity::Fresh => "fresh",
            Salinity::Salty => "salty"
        };
        builder.add("salinity", symbol_of_str(salinity));
        builder.add("quality", self.quality.into());

        add_if_positive(builder, "plants", self.plants);
        add_if_positive(builder, "rocks", self.rocks);
        add_if_positive(builder, "caves", self.caves);
    }
}

impl ToSexp for Environment {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("environment");
        self.add_to_sexp(&mut builder);
        builder.to_value()
    }
}

impl ToSexp for TankStatus {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("tank-status");

        builder.add("size", self.size.into());

        self.environment.add_to_sexp(&mut builder);

        if let Some(l) = self.lighting {
            builder.add("lighting", l.into());
        }

        if self.rounded {
            builder.add("rounded", Value::Bool(true));
        }

        builder.to_value()
    }
}
