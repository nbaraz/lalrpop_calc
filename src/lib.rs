extern crate lalrpop_util;

pub mod ast;
pub mod calc; // synthesized by LALRPOP

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::FromIterator;

    use ast;
    use calc;


    #[test]
    fn calc() {
        assert!(calc::TermParser::new().parse("22").is_ok());
        assert!(calc::TermParser::new().parse("(22)").is_ok());
        assert!(calc::TermParser::new().parse("((((22))))").is_ok());
        assert!(calc::TermParser::new().parse("((22)").is_err());
    }

    #[test]
    fn ident() {
        assert!(calc::ExprParser::new().parse("22 + 33").is_ok());

        let expr = calc::ExprParser::new().parse("aa + (22 * bb)").unwrap();
        let mut v = Vec::new();
        expr.walk(&mut |e| {
            match e {
                ast::Expr::Ident(s) => v.push(s.clone()),
                _ => (),
            };
            None::<bool>
        });
        let set = HashSet::<String>::from_iter(v);
        assert!(set.contains("aa") && set.contains("bb"));
    }
}
