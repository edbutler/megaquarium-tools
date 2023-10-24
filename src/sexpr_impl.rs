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

        builder.add("id", Value::string(self.id.clone()));
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

        builder.add("id", Value::string(self.id.clone()));
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
        let obj = expect_list_that_starts_with(value, "aquarium")?;
        let iter = expect_list(obj)?;
        let exhibits: util::Result<Vec<ExhibitDesc>> = iter.map(|x| ExhibitDesc::from_sexp(x)).collect();
        Ok(AquariumDesc { exhibits: exhibits? })
    }
}

impl ToSexp for ExhibitDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let animals = self.animals.iter().map(|e| e.to_sexp());
        sexp!((exhibit #:name ,(self.name.as_str()) #:tank ,(self.tank.to_sexp()) #:animals ,(Value::list(animals))))
    }
}

impl FromSexp for ExhibitDesc {
    fn from_sexp(value: &lexpr::Value) -> util::Result<ExhibitDesc> {
        let mut obj = expect_list_that_starts_with(value, "exhibit")?;
        let name = consume_keyword_arg(&mut obj, "name")?.as_str().ok_or(bad_sexp("expected name to be string"))?.to_string();
        let tank = TankDesc::from_sexp(consume_keyword_arg(&mut obj, "tank")?)?;
        let animal_list = consume_keyword_arg(&mut obj, "animals")?.list_iter().ok_or(bad_sexp("expected error to be list"))?;
        let animals: util::Result<Vec<AnimalDesc>> = animal_list.map(|x| AnimalDesc::from_sexp(x)).collect();
        Ok(ExhibitDesc { name, tank, animals: animals? })
    }
}

impl ToSexp for TankDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        sexp!((tank ,(self.model.clone()) ,(self.size)))
    }
}

impl FromSexp for TankDesc {
    fn from_sexp(value: &lexpr::Value) -> util::Result<TankDesc> {
        let obj = expect_list_that_starts_with(value, "tank")?;
        let (model, size) = expect_string_and_number(obj)?;
        Ok(TankDesc { model, size })
    }
}

impl ToSexp for Growth {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        match self {
            Growth::Final => sexp!((grown)),
            Growth::Growing { stage, growth } => sexp!((growing ,(*stage) ,(*growth))),
        }
    }
}

impl FromSexp for Growth {
    fn from_sexp(value: &lexpr::Value) -> util::Result<Growth> {
        let (symbol, obj) = expect_list_with_any_opening_symbol(value)?;
        match symbol {
            "grown" => Ok(Growth::Final),
            "growing" => {
                let (stage, growth) = expect_u8_and_u8(obj)?;
                Ok(Growth::Growing { stage, growth })
            },
            _ => Err(Box::new(bad_sexp("expected (grown ...) or (growing ...)")))
        }
    }
}

impl ToSexp for AnimalDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        match self {
            AnimalDesc::Summary { species, count } =>
                sexp!((animals ,(species.clone()) ,(*count))),
            AnimalDesc::Individual { species, growth } =>
                sexp!((animal ,(species.clone()) ,(growth.to_sexp()))),
        }
    }
}

impl FromSexp for AnimalDesc {
    fn from_sexp(value: &lexpr::Value) -> util::Result<AnimalDesc> {
        let (symbol, obj) = expect_list_with_any_opening_symbol(value)?;
        match symbol {
            "animals" => {
                let (species, count) = expect_string_and_number(obj)?;
                Ok(AnimalDesc::Summary { species, count })
            }
            "animal" => {
                let (species, growth_obj) = expect_string_and_any(obj)?;
                let growth = Growth::from_sexp(growth_obj)?;
                Ok(AnimalDesc::Individual { species, growth })
            },
            _ => Err(Box::new(bad_sexp("expected (animal ...) or (animals ...)")))
        }
    }
}

fn expect_list_with_any_opening_symbol<'a>(value: &'a lexpr::Value) -> util::Result<(&'a str, lexpr::cons::ListIter<'a>)> {
    match value {
        lexpr::Value::Cons(cons) => {
            match cons.car().as_symbol() {
                None => return Err(Box::new(bad_sexp(format!("expected opening symbol")))),
                Some(symbol) => {
                    match cons.cdr().list_iter() {
                        Some(iter) => Ok((symbol, iter)),
                        None => Err(Box::new(bad_sexp("expected arg to be proper list")))
                    }
                }
            }

        }
        _ => Err(Box::new(bad_sexp("expected list")))
    }
}

fn expect_list_that_starts_with<'a>(value: &'a lexpr::Value, opening_symbol: &str) -> util::Result<lexpr::cons::ListIter<'a>> {
    match value {
        lexpr::Value::Cons(cons) => {
            if cons.car().as_symbol() != Some(opening_symbol) {
                return Err(Box::new(bad_sexp(format!("expected {}", opening_symbol))));
            }

            match cons.cdr().list_iter() {
                Some(iter) => Ok(iter),
                None => Err(Box::new(bad_sexp("expected arg to be proper list")))
            }
        }
        _ => Err(Box::new(bad_sexp("expected list")))
    }
}

fn expect_list<'a>(iter: lexpr::cons::ListIter<'a>) -> util::Result<lexpr::cons::ListIter<'a>> {
    let items: Vec<&lexpr::Value> = iter.collect();
    if items.len() != 1 {
        return Err(Box::new(bad_sexp("expected call to have single argument")));
    }
    let result = items[0].list_iter().ok_or(bad_sexp("expected arg to be list"))?;
    Ok(result)
}

fn expect_two_args<'a,T,U,F1,F2>(iter: lexpr::cons::ListIter<'a>, f1:F1, f2:F2) -> util::Result<(T,U)>
        where F1:Fn(&'a Value)->util::Result<T>, F2:Fn(&'a Value)->util::Result<U> {
    let items: Vec<&lexpr::Value> = iter.collect();
    if items.len() != 2 {
        return Err(Box::new(bad_sexp(format!("expected call to have 2 arguments, got {:#?}", items))));
    }
    let x = f1(items[0])?;
    let y = f2(items[1])?;

    Ok((x, y))
}

fn expect_string_and_number(iter: lexpr::cons::ListIter<'_>) -> util::Result<(String,u16)> {
    expect_two_args(
        iter,
        |v| {
            let s = v.as_str().ok_or(bad_sexp("expected first arg to be symbol"))?;
            Ok(s.to_string())
        },
        |v| {
            let n = v.as_number().and_then(|x| x.as_u64()).ok_or(bad_sexp("expected second arg to be number"))?;
            Ok(n as u16)
        })
}

fn expect_string_and_any(iter: lexpr::cons::ListIter<'_>) -> util::Result<(String,&Value)> {
    expect_two_args(
        iter,
        |v| {
            let s = v.as_str().ok_or(bad_sexp("expected first arg to be symbol"))?;
            Ok(s.to_string())
        },
        |v| Ok(v))
}

fn expect_u8_and_u8(iter: lexpr::cons::ListIter<'_>) -> util::Result<(u8,u8)> {
    let f = |v:&Value| -> util::Result<u8> {
        let n = v.as_number().and_then(|x| x.as_u64()).ok_or(bad_sexp("expected arg to be number"))?;
        Ok(n as u8)
    };

    expect_two_args(iter, f, f)
}

fn try_consume_keyword_arg<'a>(iter: &mut lexpr::cons::ListIter<'a>, expected_keyword: &str) -> util::Result<Option<&'a lexpr::Value>> {
    match iter.next() {
        Some(lexpr::Value::Keyword(s)) if **s == *expected_keyword => (),
        _ => return Ok(None)
    };

    let result = iter.next().ok_or(bad_sexp("expected value after keyword"))?;

    Ok(Some(result))
}

fn consume_keyword_arg<'a>(iter: &mut lexpr::cons::ListIter<'a>, expected_keyword: &str) -> util::Result<&'a lexpr::Value> {
    match try_consume_keyword_arg(iter, expected_keyword)? {
        Some(s) => Ok(s),
        None => Err(Box::new(bad_sexp(format!("expected keyword {}", expected_keyword)))),
    }
}
