use anyhow::Result;
use clap::{CommandFactory, Parser as ClapParser};
use pest;
use pest::Parser;
use proptest::prelude::*;
use rand;
use rand::Rng;
use rayon::prelude::*;
use rustyline::DefaultEditor;
use std::fmt;

#[macro_use]
extern crate pest_derive;

fn main() -> Result<()> {
    let args = Cli::parse();
    if args.expression.len() == 0 {
        // start up the REPL
        let mut rl = DefaultEditor::new()?;
        println!("dice {0}", env!("CARGO_PKG_VERSION"));
        println!("enter 'help' for help, 'exit' to exit");
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    match line.as_ref() {
                        "exit" => return Ok(()),
                        "help" | "?" => {
                            let mut c = Cli::command();
                            let h = c.render_long_help();
                            c.write_long_help(&mut std::io::stdout())?;
                            println!("{}", h);
                        }
                        _ => {
                            rl.add_history_entry(line)?;
                            line.split(char::is_whitespace)
                                .filter_map(|s| match parse(s) {
                                    Ok(r) => Some(r),
                                    Err(e) => {
                                        eprintln!("{}", e);
                                        None
                                    }
                                })
                                .for_each(|r| args.print(&r, &r.roll()));
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Eof) => return Ok(()),
                Err(e) => anyhow::bail!(e),
            }
        }
    } else {
        for roll in &args.expression {
            let r = parse(roll)?;
            args.print(&r, &r.roll());
        }
    }

    return Ok(());
}

fn roll_die(size: u64) -> u64 {
    return rand::rng().random_range(1..=size);
}

proptest! {
    #[test]
    fn test_roll_sizes(size in 1..10000000) {
        let rs = roll_die(size as u64);
        assert!(rs >= 1);
        assert!(rs <= size as u64);
    }
}

/// # Rolls dice using a small expression language:
///
/// The simplest expression is just a number, indicating to roll
/// a die with that many sides, ie: `dice 20` or `dice d20` to roll a 20 sided die.
///
/// If you want to roll multiple dice you can specify how many with a prefix,
/// for example three dice with six sides each would be `3d6`.
///
/// Run without any expressions to enter interactive mode.
///
/// You can then specify how many dice to keep or drop from the roll. To drop dice
/// use `d` or `D` to drop low rolls or high rolls respectively. For example,
/// `4d6d1` says to "roll four dice with six sides dropping the lowest die", whereas
/// `2d20D1` says to "roll two dice with twenty sides each dropping the higher one".
///
/// The same thing works for keep with `k` and `K`, with `k` meaning to keep higher
/// rolls and `K` to keep lower rolls. This is different from `d` and `D`. Basically
/// it defaults (lower case) to the belief you want high rolls, therefore that is
/// easier to type (no need for the `shift` to get capital) :-) If you find it annoying
/// to use, let me know and I'll consider changing it in a future version.
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
///     2d8K1-1  2 x d8 keep the lower and subtract 1
///
#[derive(ClapParser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    /// Quiet output (just the result)
    #[structopt(short, long)]
    quiet: bool,
    /// Roll expressions, ie `4d6k3 4d6d1`
    expression: Vec<String>,
}

impl Cli {
    fn print(&self, spec: &RollSpec, roll: &Roll) {
        if self.quiet {
            println!("{}", roll.sum)
        } else {
            println!("{}\t{}", spec, roll)
        }
    }
}

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
        let mut rolls: Vec<i64> = (0..self.num)
            .into_par_iter()
            .map(|_| roll_die(self.size as u64) as i64)
            .collect();
        rolls.par_sort();

        // now that we have the rolls, figure out which to keep

        let range = if self.keep_high != 0 {
            self.num - self.keep_high..self.num
        } else if self.drop_low != 0 {
            self.drop_low..self.num
        } else if self.drop_high != 0 {
            0..self.num - self.drop_high
        } else if self.keep_low != 0 {
            0..self.keep_low
        } else {
            0..self.num
        };

        let mut sum = rolls[range].par_iter().sum();
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

#[derive(Parser)]
#[grammar = "expr.pest"]
pub struct ExprParser;

fn parse<S: Into<String>>(it: S) -> Result<RollSpec> {
    let s: &str = &it.into();
    let expr = ExprParser::parse(Rule::expression, s)
        .map_err(|e| anyhow::anyhow!("Failed to parse expression '{}': {}", s, e))?
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

    macro_rules! parse_field {
        ($field:expr, $part:expr, $err_msg:expr) => {
            $field = $part
                .as_str()
                .parse()
                .map_err(|e| anyhow::anyhow!($err_msg, $part.as_str(), e))?
        };
    }
    for part in expr.into_inner() {
        match part.as_rule() {
            Rule::n_dice => {
                parse_field!(r.num, part, "Invalid number of dice '{}': {}");
            }
            Rule::die_size => {
                parse_field!(r.size, part, "Invalid die size '{}': {}");
            }
            Rule::n_low_to_drop => {
                parse_field!(
                    r.drop_low,
                    part,
                    "Invalid number of low dice to drop '{}': {}"
                );
            }
            Rule::n_low_to_keep => {
                parse_field!(
                    r.keep_low,
                    part,
                    "Invalid number of low dice to keep '{}': {}"
                );
            }
            Rule::n_high_to_keep => {
                parse_field!(
                    r.keep_high,
                    part,
                    "Invalid number of high dice to keep '{}': {}"
                );
            }
            Rule::n_high_to_drop => {
                parse_field!(
                    r.drop_high,
                    part,
                    "Invalid number of high dice to drop '{}': {}"
                );
            }
            Rule::add_value => {
                parse_field!(r.modifier, part, "Invalid add value '{}': {}");
            }
            Rule::subtract_value => {
                r.modifier = -1
                    * part.as_str().parse::<i64>().map_err(|e| {
                        anyhow::anyhow!("Invalid subtract value '{}': {}", part.as_str(), e)
                    })?
            }
            _ => panic!("unexpected token! {}", part),
        }
    }

    return Ok(r);
}

mod test {
    use super::*;

    #[test]
    fn test_parse_3d6() {
        match parse("3d6") {
            Ok(r) => println!("roll: {}", r),
            Err(e) => eprintln!("NOOOOOO {}", e),
        }
    }

    #[test]
    fn test_parse_d6() {
        match parse("d6") {
            Ok(r) => println!("roll: {}", r),
            Err(e) => eprintln!("NOOOOOO {}", e),
        }
    }

    #[test]
    fn test_parse_6() {
        match parse("6") {
            Ok(r) => println!("roll: {}", r),
            Err(e) => eprintln!("NOOOOOO {}", e),
        }
    }

    #[test]
    fn test_parse_garbage() {
        match parse("3d8*2") {
            Ok(_r) => assert!(1 + 1 == 3),
            Err(_e) => assert!(1 + 2 == 3),
        }
    }

    #[allow(dead_code)] // used in proptest, which fools the linter
    const EXPR_PATTERN: &str = "[1-9]?{1}d[1-9]((d[1-9])|(k[1-9]))?(-[1-9])?";
    proptest! {

        #[test]
        fn test_various_parses(expr in EXPR_PATTERN) {
            match parse(expr) {
                Ok(_r) => assert!(1+1 == 2),
                Err(_e) => assert!(1+2 == 2),
            }

        }
    }
}
