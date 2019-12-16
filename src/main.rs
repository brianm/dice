use anyhow::Result;
use pest;
use rand;
use rand::Rng;
use structopt::StructOpt;

#[macro_use]
extern crate pest_derive;
use pest::Parser;

fn main() -> Result<()> {
    let args = Cli::from_args();

    for roll in &args.expression {
        let r = parse(roll)?;
        let mut sum: u64 = 0;
        for _i in 0..r.num {
            let rv = roll_die(r.size);
            sum += rv;
        }
        println!("{}d{}\t{}", r.num, r.size, sum)
    }

    return Ok(());
}

/// 
#[derive(StructOpt)]
struct Cli {
    /// A dice eexpression, such as `3d6` or `20` 
    expression: Vec<String>,
}

fn roll_die(size: u64) -> u64 {
    return rand::thread_rng().gen_range(1, size + 1);
}

#[derive(Parser)]
#[grammar = "expr.pest"]
pub struct ExprParser;

#[derive(Debug)]
struct Roll {
    num: u64,
    size: u64,
}

fn parse<S>(it: S) -> Result<Roll>
where
    S: Into<String>,
{
    let s: &str = &it.into();
    let expr = ExprParser::parse(Rule::expression, s)?.next().expect("Unable to read expression");

    let mut r = Roll { 
        num: 1, 
        size: 0,
    };

    for part in expr.into_inner() {
        match part.as_rule() {
            Rule::numberExpression => {
                r.num = part.into_inner().next().unwrap().as_str().parse()?
            }
            Rule::dieSize => {
                r.size = part.as_str().parse()?;
            }  
            _ => {}          
        }
    }

    return Ok(r);
}

#[test]
fn test_parse() {
    match parse("3d6") {
        Ok(r) => println!("roll: {:?}", r),
        Err(e) => eprintln!("NOOOOOO {}", e),
    }
}

#[test]
fn parse_thing() {
    let expr = ExprParser::parse(Rule::expr, "3d6")
        .unwrap()
        .next()
        .unwrap();
    let mut inner_rules = expr.into_inner();
    let num: u64 = inner_rules.next().unwrap().as_str().parse().unwrap();
    let size: u64 = inner_rules.next().unwrap().as_str().parse().unwrap();
    println!("{} D {}", &num, &size);
}
