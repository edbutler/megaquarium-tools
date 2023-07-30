use lexpr::*;

pub fn symbol_of_string(s:&String) -> Value {
    Value::symbol(s.clone())
}

pub fn symbol_of_str(s:&str) -> Value {
    Value::symbol(s)
}

pub trait ToSexp {
    fn to_sexp(&self) -> lexpr::Value;
}

pub struct PrettyPrinted {
    pub expr: Value,
}

fn write_indent(f: &mut std::fmt::Formatter, level: u32) -> std::fmt::Result {
    write!(f, "\n")?;
    for _ in 0..level {
        write!(f, "  ")?;
    }
    Ok(())
}

fn format(f: &mut std::fmt::Formatter, expr:&Value, indent:u32) -> std::fmt::Result {
    match expr {
        Value::Cons(cons) if indent == 0 && cons.car().is_symbol() && cons.cdr().is_list() => {
            write!(f, "({}", cons.car().as_symbol().unwrap())?;

            let mut was_kw = false;

            for x in cons.cdr().list_iter().unwrap() {
                if was_kw {
                    write!(f, " ")?;
                } else {
                    write_indent(f, indent + 1)?;
                }
                format(f, x, indent + 1)?;
                was_kw = x.is_keyword();
            }

            write!(f, ")")?;

            Ok(())
        }

        Value::Cons(_) if expr.is_list() && indent < 2 => {
            write!(f, "(")?;
            for x in expr.list_iter().unwrap() {
                write_indent(f, indent + 1)?;
                format(f, x, indent + 1)?;
            }
            write!(f, ")")?;
            Ok(())
        }

        _ => write!(f, "{}", expr)
    }
}

impl std::fmt::Display for PrettyPrinted {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        format(f, &self.expr, 0)
    }
}
