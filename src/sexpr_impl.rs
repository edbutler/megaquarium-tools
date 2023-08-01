// we have to stuff this all in here because rust formatting mangles the sexp! macro
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::tank::*;
use crate::animal::*;
use crate::aquarium::*;
use crate::sexpr_format::*;
use lexpr::*;

impl ToSexp for Species {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("species");

        builder.add("id", symbol_of_string(&self.id));
        builder.add("genus", symbol_of_string(&self.genus));

        builder.add("prey-type", symbol_of_str(self.prey_type.as_str()));

        let size = if self.immobile {
            Value::symbol("immobile")
        } else {
            Value::Number(self.maximum_size().into())
        };
        builder.add("size", size);

        if self.size.armored {
            builder.add("armored?", Value::Bool(true));
        }

        builder.add("environment", self.environment.to_sexp());

        let diet =
            match &self.diet {
                Diet::Food { food, period } => sexp!((food ,(symbol_of_string(food)) ,(*period))),
                Diet::Scavenger => sexp!((scavenger)),
                Diet::DoesNotEat => sexp!((#"no-food")),
            };
        builder.add("diet", diet);

        if let Some(s) = &self.shoaling {
            builder.add("shoaler", (*s).into());
        }

        if let Some(f) = &self.fighting {
            builder.add("fighting", invoke_symbol(f.as_str()));
        }

        if let Some(l) = &self.lighting {
            let value = match l {
                Lighting::Disallows => sexp!((#"no-light")),
                Lighting::Requires(r) => sexp!((light ,(*r)))
            };
            builder.add("light", value);
        }

        if let Some(c) = &self.cohabitation {
            builder.add("cohabitation", invoke_symbol(c.as_str()));
        }

        if self.tank.rounded_tank {
            builder.add("rounded-tank?", true.into());
        }

        if self.tank.active_swimmer {
            builder.add("active-swimmer?", true.into());
        }

        if self.predation.len() > 0 {
            let mut b = StructBuilder::new("predation");
            b.add("size", self.predation_size().into());
            let targets = self.predation.iter().map(|p| symbol_of_str(p.as_str()));
            b.add("targets", Value::list(targets));
            builder.add("predaction", b.to_value());
        }

        builder.to_value()
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

impl ToSexp for AquariumDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let exhibits = self.exhibits.iter().map(|e| e.to_sexp());
        sexp!((aquarium ,(Value::list(exhibits))))
    }
}

impl ToSexp for ExhibitDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let animals = self.animals.iter().map(|e| e.to_sexp());
        sexp!((exhibit #:tank ,(self.tank.to_sexp()) #:animals ,(Value::list(animals))))
    }
}

impl ToSexp for TankDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        sexp!((tank ,(symbol_of_string(&self.model)) ,(self.size)))
    }
}

impl ToSexp for AnimalDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        sexp!((animals ,(symbol_of_string(&self.species)) ,(self.count)))
    }
}
