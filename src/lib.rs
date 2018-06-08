extern crate lalrpop_util;
#[macro_use] extern crate failure;

pub mod ast;
pub mod execution;
pub mod calc; // synthesized by LALRPOP

#[cfg(test)]
mod tests {
    use calc;


    #[test]
    fn calc() {
        assert!(calc::TermParser::new().parse("22").is_ok());
        assert!(calc::TermParser::new().parse("(22)").is_ok());
        assert!(calc::TermParser::new().parse("((((22))))").is_ok());
        assert!(calc::TermParser::new().parse("((22)").is_err());
    }
}
