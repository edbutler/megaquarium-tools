
pub enum SExp {
    Int(i64),
    Symbol(String),
    Keyword(String, Box<SExp>),
    List(Vec<Box<SExp>>),
}

impl SExp {
    pub fn list<I>(list: I) -> SExp where I: Iterator<Item=SExp> {
        SExp::List(list.map(|x| Box::new(x)).collect())
    }

    pub fn keyword<S: Into<String>>(kw: S, arg: SExp) -> SExp {
        SExp::Keyword(kw.into(), Box::new(arg))
    }

}

impl std::fmt::Display for SExp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod test {

}