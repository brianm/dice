use anyhow::Result;
use pest;
use rand;
use rand::Rng;
use std::fmt;
use structopt::StructOpt;

#[macro_use]
extern crate pest_derive;
use pest::Parser;

fn main() -> Result<()> {
    let args = Cli::from_args();

    for roll in &args.expression {
        let r = parse(roll)?;
        println!("{}\t{}", r, r.roll())
    }

    return Ok(());
}

/// Rolls dice
#[derive(StructOpt)]
struct Cli {
    /// A dice expression, such as `3d6`, `20`, or `4d6k3+2`
    /// It is basically of the form `NdS` where N is the number of dice
    /// and S is the size of the die. This can be suffixed with the
    /// number of dice to drop (d) or keep (k), a la `4d6d1` to say
    /// "roll four dice with six sides each and drop the lowest one" or `5d8k3`
    /// to say "roll five dice with 8 sides each keeping the three highest".
    /// The expression can also, finally, be suffixed with a constant value
    /// to add or subtract, such as `2d100d1+7`
    expression: Vec<String>,
}

fn roll_die(size: u64) -> u64 {
    return rand::thread_rng().gen_range(1, size + 1);
}

#[derive(Parser)]
#[grammar = "expr.pest"]
pub struct ExprParser;

#[derive(Debug)]
struct RollSpec {
    num: usize,
    size: i64,
    keep_high: usize,
    keep_low: usize,
    drop_low: usize,
    drop_high: usize,
    modifier: i64,
}

impl fmt::Display for RollSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut suffix = String::from("");
        if self.keep_high > 0 {
            suffix.push_str(&format!(" keep highest {}", self.keep_high));
        } else if self.drop_low > 0 {
            suffix.push_str(&format!(" drop lowest {}", self.drop_low));
        } else if self.drop_high > 0 {
            suffix.push_str(&format!(" drop highest {}", self.drop_high));
        } else if self.keep_low > 0 {
            suffix.push_str(&format!(" keep lowest {}", self.keep_low));
        }

        let mut modifier = String::from("");
        if self.modifier > 0 {
            modifier.push_str(&format!(" +{}", self.modifier));
        } else if self.modifier < 0 {
            modifier.push_str(&format!(" {}", self.modifier));
        }

        write!(f, "{}d{}{}{}", self.num, self.size, suffix, modifier)
    }
}

impl RollSpec {
    fn roll(&self) -> Roll {
        let mut rolls = vec![];

        for _i in 0..self.num {
            let rv = roll_die(self.size as u64);
            rolls.push(rv as i64);
        }
        rolls.sort();
        rolls.reverse();

        // now that we have the rolls, figure out which to keep

        let range = if self.keep_high != 0 {
            0..self.keep_high
        } else if self.drop_low != 0 {
            0..self.num - self.drop_low
        } else if self.drop_high != 0 {
            self.drop_high..self.num - self.drop_low
        } else if self.keep_low != 0 {
            rolls.len() - self.keep_low..self.num - self.drop_low
        } else {
            0..self.num
        };

        let mut sum = rolls[range].iter().sum();
        sum += self.modifier;

        return Roll { rolls, sum };
    }
}

impl fmt::Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}\t{}", self.rolls, self.sum)
    }
}

struct Roll {
    rolls: Vec<i64>,
    sum: i64,
}

fn parse<S>(it: S) -> Result<RollSpec>
where
    S: Into<String>,
{
    let s: &str = &it.into();
    let expr = ExprParser::parse(Rule::expression, s)?
        .next()
        .expect("Unable to read expression");

    let mut r = RollSpec {
        num: 1,
        size: 0,
        drop_low: 0,
        drop_high: 0,
        keep_low: 0,
        keep_high: 0,
        modifier: 0,
    };

    for part in expr.into_inner() {
        match part.as_rule() {
            Rule::numberExpression => r.num = part.into_inner().next().unwrap().as_str().parse()?,
            Rule::dieSize => {
                r.size = part.as_str().parse()?;
            }
            Rule::dropKeep => {
                let sub = part.into_inner().next().unwrap();
                match sub.as_rule() {
                    Rule::numberLowOfDiceToDrop => {
                        r.drop_low = sub.as_str().parse()?;
                    }
                    Rule::numberLowOfDiceToKeep => {
                        r.keep_low = sub.as_str().parse()?;
                    }
                    Rule::numberHighOfDiceToKeep => {
                        r.keep_high = sub.as_str().parse()?;
                    }
                    Rule::numberHighOfDiceToDrop => {
                        r.drop_high = sub.as_str().parse()?;
                    }
                    _ => {
                        panic!("unexpected token! {}", sub);
                    }
                }
            }
            Rule::modifier => {
                let sub = part.into_inner().next().unwrap();
                match sub.as_rule() {
                    Rule::addValue => r.modifier = sub.as_str().parse()?,
                    Rule::subtractValue => {
                        r.modifier = -1 * sub.as_str().parse::<i64>()?;
                    }
                    _ => {
                        panic!("unexpected token! {}", sub);
                    }
                }
            }
            _ => {
                panic!("unexpected token! {}", part);
            }
        }
    }

    return Ok(r);
}

#[test]
fn test_parse() {
    match parse("3d6") {
        Ok(r) => println!("roll: {}", r),
        Err(e) => eprintln!("NOOOOOO {}", e),
    }
}
