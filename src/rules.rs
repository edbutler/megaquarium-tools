use crate::sexpr::*;
use crate::tank;
use lexpr;
use lexpr::sexp;
use Constraint::*;

pub enum Constraint {
    Temperature(tank::Temperature),
    Quality(u8),
    NeedsFood { kind: String, daily_amount: u16 },
    Scavenger,
    Shoaler(u8),
    IsBully,
    NoBully,
    NoLight,
    NeedsLight(u8),
    OnlyGenus(String),
    NoGenus(String),
    NoSpecies(String),
    NoFoodEaters(String),
    RoundedTank,
    TankSize(u16),
    Predator { kind: String, size: u16 },
}

impl Constraint {
    pub fn subsumes(&self, other: &Constraint) -> bool {
        match (self, other) {
            (Quality(x), Quality(y)) => x > y,
            (NeedsLight(x), NeedsLight(y)) => x > y,
            (TankSize(x), TankSize(y)) => x > y,
            _ => false,
        }
    }

    #[allow(unused_parens)]
    pub fn to_sexp(&self) -> lexpr::Value {
        match self {
            Temperature(t) => {
                let e = symbol_of_str(match t {
                    tank::Temperature::Cold => "cold",
                    tank::Temperature::Warm => "warm",
                });
                sexp!((temperature, e))
            }
            Quality(q) => sexp!((quality, (*q))),
            NeedsFood { kind, daily_amount } => sexp!((eats, (symbol_of_string(kind)), (*daily_amount))),
            Scavenger => sexp!((scavenger)),
            Shoaler(s) => sexp!((shoaler, (*s))),
            IsBully => sexp!((bully)),
            NoBully => sexp!((wimp)),
            NoLight => sexp!((#"no-light")),
            NeedsLight(l) => sexp!((light, (*l))),
            OnlyGenus(g) => sexp!((#"only-genus" ,(symbol_of_string(g)))),
            NoGenus(g) => sexp!((#"no-genus" ,(symbol_of_string(g)))),
            NoSpecies(s) => sexp!((#"no-species" ,(symbol_of_string(s)))),
            NoFoodEaters(e) => sexp!((#"no-food-eaters" ,(symbol_of_string(e)))),
            RoundedTank => sexp!((#"rounded-tank")),
            TankSize(s) => sexp!((#"tank-size" ,(*s))),
            Predator { kind, size } => sexp!((predator, (symbol_of_string(kind)), (*size))),
        }
    }
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Temperature(t) => write!(f, "(temperature {})", t),
            Quality(q) => write!(f, "(quality {})", q),
            NeedsFood { kind, daily_amount } => write!(f, "(eats {} {})", kind, daily_amount),
            Scavenger => write!(f, "(scavenger)"),
            Shoaler(s) => write!(f, "(shoaler {})", s),
            IsBully => write!(f, "(bully)"),
            NoBully => write!(f, "(wimp)"),
            NoLight => write!(f, "(no-light)"),
            NeedsLight(l) => write!(f, "(light {})", l),
            OnlyGenus(g) => write!(f, "(only-genus {})", g),
            NoGenus(g) => write!(f, "(no-genus {})", g),
            NoSpecies(s) => write!(f, "(no-species {})", s),
            NoFoodEaters(e) => write!(f, "(no-food-eaters {})", e),
            RoundedTank => write!(f, "(rounded-tank)"),
            TankSize(s) => write!(f, "(tank-size {})", s),
            Predator { kind, size } => write!(f, "(predator {} {})", kind, size),
        }
    }
}
