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

/// Rolls dice using a small expression language:
///
/// The simplest expression is just a number, indicating to roll
/// a die with that many sides, ie: `dice 20` to roll a 20 sided die.
///
/// If you want to roll multiple dice you can specify how many with a prefix,
/// for example three dice with six sides each would be `3d6`.
///
/// You can then specify how many dice to keep or drop from the roll. To drop dice
/// use `d` or `D` to drop low rolls or high rolls respectively. For example,
/// `4d6d1` says to "roll four dice with six sides dropping the lowest die", whereas
/// `2d20D1` says to "roll two dice with twenty sides each dropping the higher one".
///
/// The same thing works for keep with `k` and `K` saying to `k`eep the lowest or
/// `K`eep the highest.
///
/// Finally, you may add a constant modifier to the roll by appending `+` or `-` and
/// a value, such as `4d6+1` `3d6-2` or `2d20K1+7`
///
/// You can also send multiple expressions:
///
/// `dice 4d6d1 4d6d1 4d6d1 4d6d1 4d6d1 4d6d1`
///
/// In summary:
///
///     3d6      3 x d6
///
///     4d6d1    3 x d6 dropping lowest
///
///     20+1     1 x d20 and add one to the result   
///
///     2d8K1-1  2 x d8 keep the higher and subtract 1
///
#[derive(StructOpt)]
struct Cli {
    ///
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

struct Roll {
    rolls: Vec<i64>,
    sum: i64,
}

impl fmt::Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}\t{}", self.rolls, self.sum)
    }
}

fn parse<S: Into<String>>(it: S) -> Result<RollSpec> {
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
            Rule::numberOfDice => r.num = part.as_str().parse()?,
            Rule::dieSize => r.size = part.as_str().parse()?,
            Rule::numberLowOfDiceToDrop => r.drop_low = part.as_str().parse()?,
            Rule::numberLowOfDiceToKeep => r.keep_low = part.as_str().parse()?,
            Rule::numberHighOfDiceToKeep => r.keep_high = part.as_str().parse()?,
            Rule::numberHighOfDiceToDrop => r.drop_high = part.as_str().parse()?,
            Rule::addValue => r.modifier = part.as_str().parse()?,
            Rule::subtractValue => r.modifier = -1 * part.as_str().parse::<i64>()?,
            _ => panic!("unexpected token! {}", part),
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
