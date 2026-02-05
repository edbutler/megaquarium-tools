// pattern: Functional Core

// we have to stuff this all in here because rust formatting mangles the sexp! macro
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::tank::*;
use crate::animal::*;
use crate::aquarium::*;
use crate::fixture::*;
use crate::sexpr_format::*;
use crate::util;
use lexpr::*;

fn add_opt_into<I: Into<Value>>(builder: &mut StructBuilder, key: &str, x:Option<I>) {
    if let Some(v) = x {
        builder.add(key, v.into())
    }
}

fn add_opt_sexp<S: ToSexp>(builder: &mut StructBuilder, key: &str, x:Option<S>) {
    if let Some(v) = x {
        builder.add(key, v.to_sexp())
    }
}

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
            let mut b = StructBuilder::new("shoaling");
            b.add("count", s.count.into());
            if (s.one_ok) {
                b.add("oneok?", Value::Bool(true));
            }
            if (s.two_ok) {
                b.add("twook?", Value::Bool(true));
            }

            builder.add("shoaler", b.to_value());
        }

        if let Some(f) = &self.fighting {
            builder.add("fighting", invoke_symbol(f.as_str()));
        }

        if let Some(n) = &self.nibbling {
            builder.add("nibbling", invoke_symbol(n.as_str()));
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

        add_opt_into(&mut builder, "communal", self.communal);

        builder.to_value()
    }
}

impl ToSexp for Habitat {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("habitat");

        builder.add("temperature", symbol_of_str(self.temperature.as_str()));

        let salinity = match self.salinity {
            None => "both",
            Some(s) => s.as_str(),
        };
        builder.add("salinity", symbol_of_str(salinity));

        builder.add("quality", self.minimum_quality.into());

        if let Some(t) = self.interior {
            builder.add("interior", symbol_of_str(t.as_str()));
        }

        if self.active_swimmer {
            builder.add("active-swimmer?", true.into());
        }

        if self.territorial {
            builder.add("territorial?", true.into());
        }

        builder.to_value()
    }
}

impl Needs {
    #[allow(unused_parens)]
    fn try_to_sexp(&self) -> Option<lexpr::Value> {
        let mut builder: StructBuilder = StructBuilder::new("needs");

        add_opt_sexp(&mut builder, "light", self.light);
        add_opt_sexp(&mut builder, "plants", self.plants);
        add_opt_sexp(&mut builder, "rocks", self.rocks);
        add_opt_into(&mut builder, "caves", self.caves);
        add_opt_into(&mut builder, "bogwood", self.bogwood);
        add_opt_into(&mut builder, "flat-surfaces", self.flat_surfaces);
        add_opt_into(&mut builder, "vertical-surfaces", self.vertical_surfaces);
        add_opt_into(&mut builder, "fluffy-foliage", self.fluffy_foliage);
        add_opt_into(&mut builder, "open-space", self.open_space);
        add_opt_into(&mut builder, "explorer", self.explorer);

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
        if let Some(t) = self.interior {
            builder.add("interior", symbol_of_str(t.as_str()));
        }

        builder.to_value()
    }
}

impl ToSexp for FixtureModel {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("fixture-model");

        builder.add("id", Value::string(self.id.clone()));
        add_opt_into(&mut builder, "light", self.light);
        add_opt_into(&mut builder, "plants", self.plants);
        add_opt_into(&mut builder, "rocks", self.rocks);
        add_opt_into(&mut builder, "caves", self.caves);
        add_opt_into(&mut builder, "bogwood", self.bogwood);
        add_opt_into(&mut builder, "flat-surfaces", self.flat_surfaces);
        add_opt_into(&mut builder, "vertical-surfaces", self.vertical_surfaces);
        add_opt_into(&mut builder, "fluffy-foliage", self.fluffy_foliage);

        builder.to_value()
    }
}

impl ToSexp for Fixture {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        sexp!((fixture ,(self.id) ,(self.model.clone())))
    }
}

impl FromSexp for Fixture {
    fn from_sexp(value: &lexpr::Value) -> util::Result<Fixture> {
        let obj = match_list_that_starts_with(value, "fixture")?;
        let (id, model) = match_two_args(obj, match_u64, match_string)?;
        Ok(Fixture { id, model })
    }
}

impl ToSexp for FixtureDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        match self {
            FixtureDesc::Summary(FixtureCount { model, count }) =>
                sexp!((fixtures ,(model.clone()) ,(*count))),
            FixtureDesc::Individual(fixture) =>
                fixture.to_sexp(),
        }
    }
}

impl FromSexp for FixtureDesc {
    fn from_sexp(value: &lexpr::Value) -> util::Result<FixtureDesc> {
        let (symbol, obj) = match_list_with_any_opening_symbol(value)?;
        match symbol {
            "fixtures" => {
                let (model, count) = match_two_args(obj, match_string, match_u16)?;
                Ok(FixtureDesc::Summary(FixtureCount { model, count }))
            }
            "fixture" => {
                let (id, model) = match_two_args(obj, match_u64, match_string)?;
                Ok(FixtureDesc::Individual(Fixture { id, model }))
            },
            _ => Err(Box::new(bad_sexp("expected (fixture ...) or (fixtures ...)")))
        }
    }
}

impl ToSexp for Environment {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let mut builder = StructBuilder::new("environment");

        builder.add("size", self.size.into());
        builder.add("temperature", symbol_of_str(self.temperature.as_str()));
        builder.add("quality", self.quality.into());
        add_opt_into(&mut builder, "light", self.light);
        add_opt_into(&mut builder, "plants", self.plants);
        add_opt_into(&mut builder, "rocks", self.rocks);
        add_opt_into(&mut builder, "caves", self.caves);
        add_opt_into(&mut builder, "bogwood", self.bogwood);
        add_opt_into(&mut builder, "flat-surfaces", self.flat_surfaces);
        add_opt_into(&mut builder, "vertical-surfaces", self.vertical_surfaces);
        add_opt_into(&mut builder, "fluffy-foliage", self.fluffy_foliage);
        add_opt_into(&mut builder, "open-space", self.open_space);
        add_opt_into(&mut builder, "different-fixtures", self.different_decorations);
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
        let obj = match_list_that_starts_with(value, "aquarium")?;
        let iter = match_list(obj)?;
        let exhibits: util::Result<Vec<ExhibitDesc>> = iter.map(|x| ExhibitDesc::from_sexp(x)).collect();
        Ok(AquariumDesc { exhibits: exhibits? })
    }
}

impl ToSexp for ExhibitDesc {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        let animals = self.animals.iter().map(|e| e.to_sexp());
        let fixtures = self.fixtures.iter().map(|f| f.to_sexp());
        sexp!((exhibit #:name ,(self.name.as_str()) #:tank ,(self.tank.to_sexp()) #:animals ,(Value::list(animals)) #:fixtures ,(Value::list(fixtures))))
    }
}

impl FromSexp for ExhibitDesc {
    fn from_sexp(value: &lexpr::Value) -> util::Result<ExhibitDesc> {
        let mut obj = match_list_that_starts_with(value, "exhibit")?;
        let name = consume_keyword_arg(&mut obj, "name")?.as_str().ok_or(bad_sexp("expected name to be string"))?.to_string();
        let tank = Tank::from_sexp(consume_keyword_arg(&mut obj, "tank")?)?;
        let animal_list = consume_keyword_arg(&mut obj, "animals")?.list_iter().ok_or(bad_sexp("expected animals to be list"))?;
        let animals: util::Result<Vec<AnimalDesc>> = animal_list.map(|x| AnimalDesc::from_sexp(x)).collect();

        let fixtures = match try_consume_keyword_arg(&mut obj, "fixtures")? {
            Some(v) => {
                let list = v.list_iter().ok_or(bad_sexp("expected fixtures to be list"))?;
                list.map(|x| FixtureDesc::from_sexp(x)).collect::<util::Result<Vec<_>>>()?
            }
            None => vec![],
        };

        Ok(ExhibitDesc { name, tank, animals: animals?, fixtures })
    }
}

impl ToSexp for Tank {
    #[allow(unused_parens)]
    fn to_sexp(&self) -> lexpr::Value {
        sexp!((tank ,(self.id) ,(self.model.clone()) (size ,(self.size.0) ,(self.size.1))))
    }
}

impl FromSexp for Tank {
    fn from_sexp(value: &lexpr::Value) -> util::Result<Tank> {
        fn match_size(v: &Value) -> util::Result<(u16, u16)> {
            let o = match_list_that_starts_with(v, "size")?;
            match_two_args(o, match_u16, match_u16)
        }

        let obj = match_list_that_starts_with(value, "tank")?;
        let (id, model, size) = match_three_args(obj, match_u64, match_string, match_size)?;
        Ok(Tank { id, model, size })
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
        let (symbol, obj) = match_list_with_any_opening_symbol(value)?;
        match symbol {
            "grown" => Ok(Growth::Final),
            "growing" => {
                let (stage, growth) = match_two_args(obj, match_u8, match_u8)?;
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
            AnimalDesc::Summary(SpeciesCount { species, count }) =>
                sexp!((animals ,(species.clone()) ,(*count))),
            AnimalDesc::Individual(Animal { id,  species, growth }) =>
                sexp!((animal ,(*id) ,(species.clone()) ,(growth.to_sexp()))),
        }
    }
}

impl FromSexp for AnimalDesc {
    fn from_sexp(value: &lexpr::Value) -> util::Result<AnimalDesc> {
        let (symbol, obj) = match_list_with_any_opening_symbol(value)?;
        match symbol {
            "animals" => {
                let (species, count) = match_two_args(obj, match_string, match_u16)?;
                Ok(AnimalDesc::Summary(SpeciesCount { species, count }))
            }
            "animal" => {
                let (id, species, growth) = match_three_args(obj, match_u64, match_string, Growth::from_sexp)?;
                Ok(AnimalDesc::Individual(Animal { id, species, growth }))
            },
            _ => Err(Box::new(bad_sexp("expected (animal ...) or (animals ...)")))
        }
    }
}

fn match_list_with_any_opening_symbol<'a>(value: &'a lexpr::Value) -> util::Result<(&'a str, lexpr::cons::ListIter<'a>)> {
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

fn match_list_that_starts_with<'a>(value: &'a lexpr::Value, opening_symbol: &str) -> util::Result<lexpr::cons::ListIter<'a>> {
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
        _ => Err(Box::new(bad_sexp(format!("expected list starting with {}", opening_symbol))))
    }
}

fn match_list<'a>(iter: lexpr::cons::ListIter<'a>) -> util::Result<lexpr::cons::ListIter<'a>> {
    let items: Vec<&lexpr::Value> = iter.collect();
    if items.len() != 1 {
        return Err(Box::new(bad_sexp("expected call to have single argument")));
    }
    let result = items[0].list_iter().ok_or(bad_sexp("expected arg to be list"))?;
    Ok(result)
}

trait ParseFn<T> = Fn(& Value)->util::Result<T>;

fn match_two_args<'a,T,U,F1,F2>(iter: lexpr::cons::ListIter<'a>, f1:F1, f2:F2) -> util::Result<(T,U)>
        where F1:ParseFn<T>, F2:ParseFn<U> {
    let items: Vec<&lexpr::Value> = iter.collect();
    if items.len() != 2 {
        return Err(Box::new(bad_sexp(format!("expected call to have 2 arguments, got {:#?}", items))));
    }
    let x = f1(items[0])?;
    let y = f2(items[1])?;

    Ok((x, y))
}

fn match_three_args<'a,T,U,V,F1,F2,F3>(iter: lexpr::cons::ListIter<'a>, f1:F1, f2:F2, f3:F3) -> util::Result<(T,U,V)>
        where F1:ParseFn<T>, F2:ParseFn<U>, F3:ParseFn<V> {
    let items: Vec<&lexpr::Value> = iter.collect();
    if items.len() != 3 {
        return Err(Box::new(bad_sexp(format!("expected call to have 3 arguments, got {:#?}", items))));
    }
    let x = f1(items[0])?;
    let y = f2(items[1])?;
    let z = f3(items[2])?;

    Ok((x, y, z))
}

fn match_u8(v: &Value) -> util::Result<u8> {
    let n = v.as_number().and_then(|x| x.as_u64()).ok_or(bad_sexp("expected arg to be u8"))?;
    Ok(n as u8)
}

fn match_u16(v: &Value) -> util::Result<u16> {
    let n = v.as_number().and_then(|x| x.as_u64()).ok_or(bad_sexp("expected arg to be u16"))?;
    Ok(n as u16)
}

fn match_u64(v: &Value) -> util::Result<u64> {
    let n = v.as_number().and_then(|x| x.as_u64()).ok_or(bad_sexp("expected arg to be u64"))?;
    Ok(n)
}

fn match_string(v: &Value) -> util::Result<String> {
    let s = v.as_str().ok_or(bad_sexp("expected arg to be symbol"))?;
    Ok(s.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: serialize to sexp, parse back, serialize again - strings should match
    fn roundtrip_string<T: ToSexp + FromSexp>(value: &T) -> String {
        let sexp = value.to_sexp();
        let parsed = T::from_sexp(&sexp).expect("roundtrip parse failed");
        parsed.to_sexp().to_string()
    }

    // === Round-trip tests ===

    #[test]
    fn test_fixture_roundtrip() {
        let fixture = FixtureDesc::Individual(Fixture { id: 42, model: "coral_rock".to_string() });
        let original = fixture.to_sexp().to_string();
        let roundtripped = roundtrip_string(&fixture);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_fixture_desc_summary_roundtrip() {
        let desc = FixtureDesc::Summary(FixtureCount { model: "live_rock".to_string(), count: 3 });
        let original = desc.to_sexp().to_string();
        let roundtripped = roundtrip_string(&desc);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_tank_roundtrip() {
        let tank = Tank { id: 1, model: "glass_tank".to_string(), size: (5, 3) };
        let original = tank.to_sexp().to_string();
        let roundtripped = roundtrip_string(&tank);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_growth_final_roundtrip() {
        let growth = Growth::Final;
        let original = growth.to_sexp().to_string();
        let roundtripped = roundtrip_string(&growth);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_growth_growing_roundtrip() {
        let growth = Growth::Growing { stage: 2, growth: 15 };
        let original = growth.to_sexp().to_string();
        let roundtripped = roundtrip_string(&growth);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_animal_desc_summary_roundtrip() {
        let desc = AnimalDesc::Summary(SpeciesCount { species: "clownfish".to_string(), count: 5 });
        let original = desc.to_sexp().to_string();
        let roundtripped = roundtrip_string(&desc);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_animal_desc_individual_final_roundtrip() {
        let desc = AnimalDesc::Individual(Animal { id: 99, species: "angelfish".to_string(), growth: Growth::Final });
        let original = desc.to_sexp().to_string();
        let roundtripped = roundtrip_string(&desc);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_animal_desc_individual_growing_roundtrip() {
        let desc = AnimalDesc::Individual(Animal { id: 7, species: "guppy".to_string(), growth: Growth::Growing { stage: 1, growth: 8 } });
        let original = desc.to_sexp().to_string();
        let roundtripped = roundtrip_string(&desc);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_exhibit_desc_roundtrip_minimal() {
        let exhibit = ExhibitDesc {
            name: "Reef Tank".to_string(),
            tank: Tank { id: 1, model: "basic_tank".to_string(), size: (4, 4) },
            animals: vec![AnimalDesc::Summary(SpeciesCount { species: "neon_tetra".to_string(), count: 10 })],
            fixtures: vec![],
        };
        let original = exhibit.to_sexp().to_string();
        let roundtripped = roundtrip_string(&exhibit);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_exhibit_desc_roundtrip_with_fixtures() {
        let exhibit = ExhibitDesc {
            name: "Coral Display".to_string(),
            tank: Tank { id: 2, model: "large_tank".to_string(), size: (8, 6) },
            animals: vec![
                AnimalDesc::Summary(SpeciesCount { species: "clownfish".to_string(), count: 2 }),
                AnimalDesc::Individual(Animal { id: 5, species: "angelfish".to_string(), growth: Growth::Final }),
            ],
            fixtures: vec![
                FixtureDesc::Individual(Fixture { id: 10, model: "live_rock".to_string() }),
                FixtureDesc::Individual(Fixture { id: 11, model: "anemone".to_string() }),
            ],
        };
        let original = exhibit.to_sexp().to_string();
        let roundtripped = roundtrip_string(&exhibit);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_aquarium_desc_roundtrip_empty() {
        let aquarium = AquariumDesc { exhibits: vec![] };
        let original = aquarium.to_sexp().to_string();
        let roundtripped = roundtrip_string(&aquarium);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_aquarium_desc_roundtrip_single_exhibit() {
        let aquarium = AquariumDesc {
            exhibits: vec![ExhibitDesc {
                name: "Main Tank".to_string(),
                tank: Tank { id: 1, model: "display_tank".to_string(), size: (10, 5) },
                animals: vec![AnimalDesc::Summary(SpeciesCount { species: "goldfish".to_string(), count: 3 })],
                fixtures: vec![FixtureDesc::Individual(Fixture { id: 1, model: "plant".to_string() })],
            }],
        };
        let original = aquarium.to_sexp().to_string();
        let roundtripped = roundtrip_string(&aquarium);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_aquarium_desc_roundtrip_multiple_exhibits() {
        let aquarium = AquariumDesc {
            exhibits: vec![
                ExhibitDesc {
                    name: "Tropical".to_string(),
                    tank: Tank { id: 1, model: "tank_a".to_string(), size: (5, 5) },
                    animals: vec![AnimalDesc::Summary(SpeciesCount { species: "guppy".to_string(), count: 6 })],
                    fixtures: vec![],
                },
                ExhibitDesc {
                    name: "Coldwater".to_string(),
                    tank: Tank { id: 2, model: "tank_b".to_string(), size: (6, 4) },
                    animals: vec![AnimalDesc::Individual(Animal { id: 10, species: "trout".to_string(), growth: Growth::Final })],
                    fixtures: vec![FixtureDesc::Individual(Fixture { id: 20, model: "rock".to_string() })],
                },
            ],
        };
        let original = aquarium.to_sexp().to_string();
        let roundtripped = roundtrip_string(&aquarium);
        assert_eq!(original, roundtripped);
    }

    // === ToSexp snapshot tests ===

    #[test]
    fn test_need_dislikes_to_sexp() {
        let need = Need::Dislikes;
        let result = need.to_sexp().to_string();
        assert_eq!(result, "(dislikes)");
    }

    #[test]
    fn test_need_loves_to_sexp() {
        let need = Need::Loves(5);
        let result = need.to_sexp().to_string();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_habitat_minimal_to_sexp() {
        let habitat = Habitat {
            temperature: Temperature::Warm,
            salinity: None,
            minimum_quality: 50,
            interior: None,
            active_swimmer: false,
            territorial: false,
        };
        let result = habitat.to_sexp().to_string();
        assert_eq!(result, "(habitat #:temperature warm #:salinity both #:quality 50)");
    }

    #[test]
    fn test_habitat_full_to_sexp() {
        let habitat = Habitat {
            temperature: Temperature::Cold,
            salinity: Some(Salinity::Fresh),
            minimum_quality: 75,
            interior: Some(Interior::Rounded),
            active_swimmer: true,
            territorial: true,
        };
        let result = habitat.to_sexp().to_string();
        assert_eq!(result, "(habitat #:temperature cold #:salinity fresh #:quality 75 #:interior rounded #:active-swimmer? #t #:territorial? #t)");
    }

    #[test]
    fn test_environment_minimal_to_sexp() {
        let env = Environment {
            size: 100,
            temperature: Temperature::Warm,
            salinity: Salinity::Salty,
            quality: 60,
            light: None,
            plants: None,
            rocks: None,
            caves: None,
            bogwood: None,
            flat_surfaces: None,
            vertical_surfaces: None,
            fluffy_foliage: None,
            interior: None,
            open_space: None,
            different_decorations: None,
        };
        let result = env.to_sexp().to_string();
        assert_eq!(result, "(environment #:size 100 #:temperature warm #:quality 60)");
    }

    #[test]
    fn test_environment_full_to_sexp() {
        let env = Environment {
            size: 200,
            temperature: Temperature::Cold,
            salinity: Salinity::Fresh,
            quality: 80,
            light: Some(5),
            plants: Some(10),
            rocks: Some(8),
            caves: Some(3),
            bogwood: Some(2),
            flat_surfaces: Some(4),
            vertical_surfaces: Some(6),
            fluffy_foliage: Some(1),
            interior: Some(Interior::Kreisel),
            open_space: Some(7),
            different_decorations: Some(9),
        };
        let result = env.to_sexp().to_string();
        assert_eq!(result, "(environment #:size 200 #:temperature cold #:quality 80 #:light 5 #:plants 10 #:rocks 8 #:caves 3 #:bogwood 2 #:flat-surfaces 4 #:vertical-surfaces 6 #:fluffy-foliage 1 #:open-space 7 #:different-fixtures 9 #:interior kreisel)");
    }

    #[test]
    fn test_tank_model_minimal_to_sexp() {
        let model = TankModel {
            id: "basic_glass".to_string(),
            min_size: (2, 2),
            max_size: (10, 10),
            double_density: 4,
            interior: None,
        };
        let result = model.to_sexp().to_string();
        assert_eq!(result, "(tank-model #:id \"basic_glass\" #:min-size (2 . 2) #:max-size (10 . 10) #:density 2.0)");
    }

    #[test]
    fn test_tank_model_with_interior_to_sexp() {
        let model = TankModel {
            id: "kreisel_tank".to_string(),
            min_size: (3, 3),
            max_size: (6, 6),
            double_density: 7,
            interior: Some(Interior::Kreisel),
        };
        let result = model.to_sexp().to_string();
        assert_eq!(result, "(tank-model #:id \"kreisel_tank\" #:min-size (3 . 3) #:max-size (6 . 6) #:density 3.5 #:interior kreisel)");
    }

    #[test]
    fn test_fixture_model_minimal_to_sexp() {
        let model = FixtureModel {
            id: "empty_rock".to_string(),
            light: None,
            plants: None,
            rocks: None,
            caves: None,
            bogwood: None,
            flat_surfaces: None,
            vertical_surfaces: None,
            fluffy_foliage: None,
        };
        let result = model.to_sexp().to_string();
        assert_eq!(result, "(fixture-model #:id \"empty_rock\")");
    }

    #[test]
    fn test_fixture_model_full_to_sexp() {
        let model = FixtureModel {
            id: "deluxe_decoration".to_string(),
            light: Some(3),
            plants: Some(5),
            rocks: Some(4),
            caves: Some(2),
            bogwood: Some(1),
            flat_surfaces: Some(6),
            vertical_surfaces: Some(7),
            fluffy_foliage: Some(8),
        };
        let result = model.to_sexp().to_string();
        assert_eq!(result, "(fixture-model #:id \"deluxe_decoration\" #:light 3 #:plants 5 #:rocks 4 #:caves 2 #:bogwood 1 #:flat-surfaces 6 #:vertical-surfaces 7 #:fluffy-foliage 8)");
    }

    #[test]
    fn test_species_minimal_to_sexp() {
        let species = Species {
            id: "test_fish".to_string(),
            genus: "Testus".to_string(),
            prey_type: PreyType::Fish,
            size: Size { stages: vec![], final_size: 5, armored: false, immobile: false },
            habitat: Habitat {
                temperature: Temperature::Warm,
                salinity: Some(Salinity::Salty),
                minimum_quality: 50,
                interior: None,
                active_swimmer: false,
                territorial: false,
            },
            diet: Diet::DoesNotEat,
            greedy: false,
            needs: Needs {
                light: None, plants: None, rocks: None, caves: None,
                bogwood: None, flat_surfaces: None, vertical_surfaces: None,
                fluffy_foliage: None, open_space: None, explorer: None,
            },
            shoaling: None,
            fighting: None,
            nibbling: None,
            cohabitation: None,
            predation: vec![],
            communal: None,
            breeding: Breeding::CannotBread,
        };
        let result = species.to_sexp().to_string();
        assert_eq!(result, "(species #:id \"test_fish\" #:genus Testus #:prey-type fish #:size 5 #:habitat (habitat #:temperature warm #:salinity salty #:quality 50) #:diet (no-food))");
    }

    #[test]
    fn test_species_with_diet_food_to_sexp() {
        let species = Species {
            id: "hungry_fish".to_string(),
            genus: "Hungrius".to_string(),
            prey_type: PreyType::Fish,
            size: Size { stages: vec![], final_size: 8, armored: false, immobile: false },
            habitat: Habitat {
                temperature: Temperature::Cold,
                salinity: Some(Salinity::Fresh),
                minimum_quality: 60,
                interior: None,
                active_swimmer: false,
                territorial: false,
            },
            diet: Diet::Food { food: "flakes".to_string(), period: 3 },
            greedy: true,
            needs: Needs {
                light: None, plants: None, rocks: None, caves: None,
                bogwood: None, flat_surfaces: None, vertical_surfaces: None,
                fluffy_foliage: None, open_space: None, explorer: None,
            },
            shoaling: None,
            fighting: None,
            nibbling: None,
            cohabitation: None,
            predation: vec![],
            communal: None,
            breeding: Breeding::CannotBread,
        };
        let result = species.to_sexp().to_string();
        assert_eq!(result, "(species #:id \"hungry_fish\" #:genus Hungrius #:prey-type fish #:size 8 #:habitat (habitat #:temperature cold #:salinity fresh #:quality 60) #:diet (food flakes 3) #:greedy #t)");
    }

    #[test]
    fn test_species_with_diet_scavenger_to_sexp() {
        let species = Species {
            id: "bottom_feeder".to_string(),
            genus: "Scavengus".to_string(),
            prey_type: PreyType::Crustacean,
            size: Size { stages: vec![], final_size: 4, armored: true, immobile: false },
            habitat: Habitat {
                temperature: Temperature::Warm,
                salinity: None,
                minimum_quality: 40,
                interior: None,
                active_swimmer: false,
                territorial: false,
            },
            diet: Diet::Scavenger,
            greedy: false,
            needs: Needs {
                light: None, plants: None, rocks: None, caves: None,
                bogwood: None, flat_surfaces: None, vertical_surfaces: None,
                fluffy_foliage: None, open_space: None, explorer: None,
            },
            shoaling: None,
            fighting: None,
            nibbling: None,
            cohabitation: None,
            predation: vec![],
            communal: None,
            breeding: Breeding::CannotBread,
        };
        let result = species.to_sexp().to_string();
        assert_eq!(result, "(species #:id \"bottom_feeder\" #:genus Scavengus #:prey-type crustacean #:size 4 #:armored? #t #:habitat (habitat #:temperature warm #:salinity both #:quality 40) #:diet (scavenger))");
    }

    #[test]
    fn test_species_full_to_sexp() {
        let species = Species {
            id: "complex_fish".to_string(),
            genus: "Complexus".to_string(),
            prey_type: PreyType::Fish,
            size: Size { stages: vec![], final_size: 10, armored: false, immobile: false },
            habitat: Habitat {
                temperature: Temperature::Warm,
                salinity: Some(Salinity::Salty),
                minimum_quality: 70,
                interior: Some(Interior::Rounded),
                active_swimmer: true,
                territorial: true,
            },
            diet: Diet::Food { food: "pellets".to_string(), period: 2 },
            greedy: true,
            needs: Needs {
                light: Some(Need::Loves(3)),
                plants: Some(Need::Dislikes),
                rocks: Some(Need::Loves(5)),
                caves: Some(2),
                bogwood: Some(1),
                flat_surfaces: Some(3),
                vertical_surfaces: Some(4),
                fluffy_foliage: Some(2),
                open_space: Some(5),
                explorer: Some(3),
            },
            shoaling: Some(Shoaling { count: 5, one_ok: true, two_ok: false }),
            fighting: Some(Fighting::Bully),
            nibbling: Some(Nibbling::Nibbler),
            cohabitation: Some(Cohabitation::OnlyCongeners),
            predation: vec![PreyType::Crustacean, PreyType::Baby],
            communal: Some(4),
            breeding: Breeding::CannotBread,
        };
        let result = species.to_sexp().to_string();
        assert!(result.contains("#:id \"complex_fish\""));
        assert!(result.contains("#:shoaler (shoaling #:count 5 #:oneok? #t)"));
        assert!(result.contains("#:fighting (bully)"));
        assert!(result.contains("#:nibbling (nibbler)"));
        assert!(result.contains("#:cohabitation (only-congeners)"));
        assert!(result.contains("#:predaction (predation #:size 4 #:targets (crustacean baby))"));
        assert!(result.contains("#:communal 4"));
        assert!(result.contains("#:needs (needs #:light 3 #:plants (dislikes) #:rocks 5 #:caves 2 #:bogwood 1 #:flat-surfaces 3 #:vertical-surfaces 4 #:fluffy-foliage 2 #:open-space 5 #:explorer 3)"));
    }
}
