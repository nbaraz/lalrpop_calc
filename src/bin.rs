extern crate lalrpop_calc;

use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead, BufReader};

use lalrpop_calc::{ast, calc, execution};
use ast::Statement;
use execution::resolve;

fn main() -> Result<(), Box<Error>> {
    let stdin = io::stdin();
    let input = BufReader::new(stdin.lock());

    let mut vars = HashMap::new();
    for line in input.lines() {
        let line = line?;
        line.trim();
        if line.len() == 0 {
            continue;
        }

        let stmt = match calc::StatementParser::new().parse(&line) {
            Ok(stmt) => stmt,
            Err(e) => {
                println!("Invalid statement: {}", e);
                continue;
            }
        };

        match stmt {
            Statement::Assign(name, expr) => {
                vars.insert(name, expr);
            }
            Statement::Print(expr) => {
                match resolve(&expr, &vars) {
                    Ok(val) => {
                        println!("{}", val);
                    }
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };
            }
            Statement::Repr(expr) => {
                println!("{}", ast::repr_expr(&expr, &vars));
            }
        };
    }

    Ok(())
}
