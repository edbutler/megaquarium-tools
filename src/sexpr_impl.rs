// we have to stuff this all in here because rust formatting mangles the sexp! macro
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::tank::*;
use crate::animal::*;
use crate::aquarium::*;
use crate::sexpr_format::*;
use crate::util;
use lexpr::*;

impl ToSexp for Species {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("species");

        builder.add("id", symbol_of_string(&self.id));
        builder.add("genus", symbol_of_string(&self.genus));

        builder.add("prey-type", symbol_of_str(self.prey_type.as_str()));

        let size = if self.size.immobile {
            Value::symbol("immobile")
        } else {
            Value::Number(self.maximum_size().into())
        };
        builder.add("size", size);

        if self.size.armored {
            builder.add("armored?", Value::Bool(true));
        }

        builder.add("habitat", self.habitat.to_sexp());

        let diet =
            match &self.diet {
                Diet::Food { food, period } => sexp!((food ,(symbol_of_string(food)) ,(*period))),
                Diet::Scavenger => sexp!((scavenger)),
                Diet::DoesNotEat => sexp!((#"no-food")),
            };
        builder.add("diet", diet);

        if self.greedy {
            builder.add("greedy", Value::Bool(true));
        }

        if let Some(v) = self.needs.try_to_sexp() {
            builder.add("needs", v);
        }

        if let Some(s) = &self.shoaling {
            builder.add("shoaler", (*s).into());
        }

        if let Some(f) = &self.fighting {
            builder.add("fighting", invoke_symbol(f.as_str()));
        }

        if let Some(c) = &self.cohabitation {
            builder.add("cohabitation", invoke_symbol(c.as_str()));
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

impl ToSexp for Habitat {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("habitat");

        builder.add("temperature", symbol_of_str(self.temperature.as_str()));

        builder.add("quality", self.minimum_quality.into());

        if let Some(t) = self.interior {
            builder.add("interior", symbol_of_str(t.as_str()));
        }

        if self.active_swimmer {
            builder.add("active-swimmer?", true.into());
        }

        builder.to_value()
    }
}

impl Needs {
    #[allow(unused_parens)]
    fn try_to_sexp(&self) -> Option<lexpr::Value> {
        let mut builder: StructBuilder = StructBuilder::new("needs");

        if let Some(p) = self.plants {
            builder.add("plants", p.to_sexp());
        }
        if let Some(r) = self.rocks {
            builder.add("rocks", r.to_sexp());
        }
        if let Some(c) = self.caves {
            builder.add("caves", c.into());
        }
        if let Some(l) = self.light {
            builder.add("light", l.to_sexp());
        }

        if builder.added > 0 {
            Some(builder.to_value())
        } else {
            None
        }
    }
}

impl ToSexp for Need {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        match self {
            Need::Dislikes => sexp!((#"dislikes")),
            Need::Loves(r) => (*r).into()
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

fn add_if_some<I: Into<Value>>(builder: &mut StructBuilder, key: &str, x:Option<I>) {
    if let Some(v) = x {
        builder.add(key, v.into())
    }
}

impl ToSexp for Environment {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("environment");

        builder.add("size", self.size.into());
        builder.add("temperature", symbol_of_str(self.temperature.as_str()));
        builder.add("quality", self.quality.into());
        add_if_some(&mut builder, "plants", self.plants);
        add_if_some(&mut builder, "rocks", self.rocks);
        add_if_some(&mut builder, "caves", self.caves);
        if let Some(t) = self.interior {
            builder.add("interior", symbol_of_str(t.as_str()));
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

impl FromSexp for AquariumDesc {
    fn from_sexp(value: &lexpr::Value) -> util::Result<AquariumDesc> {
        match value {
            lexpr::Value::Cons(cons) => {
                if cons.car().as_symbol() != Some("aquarium") {
                    return Err(Box::new(bad_sexp("expected aquarium")))
                }

                Ok(AquariumDesc { exhibits: vec![] })
            }
            _ => Err(Box::new(bad_sexp("expected list")))
        }
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
