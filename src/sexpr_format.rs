use lexpr::*;

pub fn symbol_of_string(s:&String) -> Value {
    Value::symbol(s.clone())
}

pub fn symbol_of_str(s:&str) -> Value {
    Value::symbol(s)
}

#[allow(unused_parens)]
pub fn invoke_symbol(s:&str) -> Value {
    sexp!((,(symbol_of_str(s))))
}

pub trait ToSexp {
    fn to_sexp(&self) -> lexpr::Value;
}

pub struct PrettyPrinted {
    pub expr: Value,
}

pub struct StructBuilder {
    list: Vec<Value>,
    pub added: u32,
}

impl StructBuilder {
    pub fn new(id: &str) -> StructBuilder {
        let list = vec![Value::symbol(id)];
        StructBuilder { list, added: 0 }
    }

    pub fn add(&mut self, kw: &str, value: Value) {
        self.list.push(Value::keyword(kw));
        self.list.push(value);
        self.added += 1;
    }

    pub fn to_value(&self) -> Value {
        Value::list(self.list.clone())
    }
}

fn write_indent(f: &mut std::fmt::Formatter, level: u32) -> std::fmt::Result {
    write!(f, "\n")?;
    for _ in 0..level {
        write!(f, "  ")?;
    }
    Ok(())
}

fn is_call(cons: &Cons) -> bool {
    cons.car().is_symbol() && cons.cdr().is_list()
}

fn should_show_call_on_multiple_lines(cons: &Cons, indent:u32) -> bool {
    indent < 3 && cons.cdr().list_iter().unwrap().any(|x| x.is_keyword())
}

fn format(f: &mut std::fmt::Formatter, expr:&Value, indent:u32) -> std::fmt::Result {
    match expr {
        Value::Cons(cons) if is_call(cons) && should_show_call_on_multiple_lines(cons, indent) => {
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

        Value::Cons(cons) if is_call(cons) => {
            write!(f, "({}", cons.car().as_symbol().unwrap())?;

            for x in cons.cdr().list_iter().unwrap() {
                write!(f, " ")?;
                format(f, x, indent)?;
            }

            write!(f, ")")?;
            Ok(())
        }

        Value::Cons(cons) if expr.is_list() && !cons.car().is_symbol() && indent < 4 => {
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
