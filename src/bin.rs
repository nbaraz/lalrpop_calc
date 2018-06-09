extern crate lalrpop_calc;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{self, BufRead, BufReader};

use ast::Statement;
use execution::{resolve, resolve_initial};
use lalrpop_calc::{ast, calc, execution};

fn interpret_input<T: std::io::Read>(input: BufReader<T>) -> Result<(), Box<Error>> {
    let mut vars = HashMap::new();
    for line in input.lines() {
        let line = line?;
        line.trim();
        if line.len() == 0 {
            continue;
        }

        let mut stmt = match calc::StatementParser::new().parse(&line) {
            Ok(stmt) => stmt,
            Err(e) => {
                println!("Invalid statement: {}", e);
                continue;
            }
        };

        match stmt {
            Statement::Assign(name, mut expr) => {
                if let Err(e) = resolve_initial(&mut expr, &vars) {
                    println!("{}", e);
                    continue;
                }
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
            Statement::Repr(repr_mode, expr) => {
                println!("{}", ast::repr_expr(&expr, &vars, repr_mode));
            }
        };
    }

    Ok(())
}

fn main() -> Result<(), Box<Error>> {
    if let Some(f) = env::args().nth(1) {
        println!("Executing lines from {}", f);
        let input = BufReader::new(std::fs::File::open(f)?);
        interpret_input(input)
    } else {
        let stdin = io::stdin();
        let input = BufReader::new(stdin.lock());
        interpret_input(input)
    }
}
