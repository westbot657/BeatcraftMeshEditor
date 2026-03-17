use std::collections::HashMap;
use std::iter::Peekable;

use indexmap::IndexMap;
use regex::Matches;


pub trait MapIndexable {
    fn get(&self, key: &str) -> Option<&f32>;
    fn get_mut(&mut self, key: &str) -> Option<&mut f32>;
    fn insert(&mut self, key: String, val: f32);
}

impl MapIndexable for HashMap<String, f32> {
    fn get(&self, key: &str) -> Option<&f32> {
        HashMap::get(self, key)
    }
    fn get_mut(&mut self, key: &str) -> Option<&mut f32> {
        HashMap::get_mut(self, key)
    }
    fn insert(&mut self, key: String, val: f32) {
        HashMap::insert(self, key, val);
    }
}

impl MapIndexable for IndexMap<String, f32> {
    fn get(&self, key: &str) -> Option<&f32> {
        IndexMap::get(self, key)
    }
    fn get_mut(&mut self, key: &str) -> Option<&mut f32> {
        IndexMap::get_mut(self, key)
    }
    fn insert(&mut self, key: String, val: f32) {
        IndexMap::insert(self, key, val);
    }
}



pub fn eval(expr: &str, var_table: &mut impl MapIndexable) -> Option<f32> {
    let splitter = regex::Regex::new(r#"((\d*\.\d+|\d+\.\d*|\d+|\w+)|[/()*+\-%,])"#).unwrap();

    let mut tokens = splitter.find_iter(expr).peekable();

    let res = eval_expr(&mut tokens, var_table)?;
    if tokens.next().is_some() || res.is_nan() {
        None
    } else {
        Some(res)
    }
}

type PeekMatch<'a> = Peekable<Matches<'a, 'a>>;

fn eval_expr(tokens: &mut PeekMatch<'_>, vars: &mut impl MapIndexable) -> Option<f32> {
    let mut left = eval_term(tokens, vars)?;
    while let Some(token) = tokens.peek() {
        match token.as_str() {
            "+" => left += { tokens.next(); eval_term(tokens, vars)? },
            "-" => left -= { tokens.next(); eval_term(tokens, vars)? },
            _ => break,
        }
    }
    Some(left)
}

fn eval_term(tokens: &mut PeekMatch<'_>, vars: &mut impl MapIndexable) -> Option<f32> {
    let mut left = eval_factor(tokens, vars)?;
    while let Some(token) = tokens.peek() {
        match token.as_str() {
            "*" => left *= { tokens.next(); eval_factor(tokens, vars)? },
            "/" => left /= { tokens.next(); eval_factor(tokens, vars)? },
            "%" => left %= { tokens.next(); eval_factor(tokens, vars)? },
            _ => break,
        }
    }
    Some(left)
}

fn eval_factor(tokens: &mut PeekMatch<'_>, vars: &mut impl MapIndexable) -> Option<f32> {
    if let Some(token) = tokens.peek() {
        let tk = token.as_str();
        if let Ok(val) = tk.parse::<f32>() {
            tokens.next();
            return Some(val)
        }

        match tk {
            "-" => { tokens.next()?; Some(-eval_factor(tokens, vars)?) }
            "pi" => { tokens.next()?; Some(std::f32::consts::PI) },
            "e" => { tokens.next()?; Some(std::f32::consts::E) },

            "cos"   => eval_fn(|args| Some(args.first()?.cos()),               tokens, vars),
            "sin"   => eval_fn(|args| Some(args.first()?.sin()),               tokens, vars),
            "tan"   => eval_fn(|args| Some(args.first()?.tan()),               tokens, vars),
            "sqrt"  => eval_fn(|args| Some(args.first()?.sqrt()),              tokens, vars),
            "pow"   => eval_fn(|args| Some(args.first()?.powf(*args.get(1)?)), tokens, vars),
            "sign"  => eval_fn(|args| Some(args.first()?.signum()),            tokens, vars),
            "abs"   => eval_fn(|args| Some(args.first()?.abs()),               tokens, vars),
            "floor" => eval_fn(|args| Some(args.first()?.floor()),             tokens, vars),
            "ceil"  => eval_fn(|args| Some(args.first()?.ceil()),              tokens, vars),
            "ln"    => eval_fn(|args| Some(args.first()?.ln()),                tokens, vars),

            "(" => {
                let _ = tokens.next()?;

                let exp = eval_expr(tokens, vars)?;

                if let Some(t) = tokens.next() && t.as_str() == ")" {
                    Some(exp)
                } else {
                    None
                }
            }

            _ => {
                let token = tokens.next()?;
                let tk = token.as_str().to_string();

                if let Some(next) = tokens.peek() && next.as_str() == "=" {
                    tokens.next();
                    let assign = eval_expr(tokens, vars)?;
                    if let Some(val) = vars.get_mut(&tk) {
                        *val = assign;
                    } else {
                        vars.insert(tk.clone(), assign);
                    }
                }
                vars.get(&tk).copied()

            }
        }

    } else { None }
}

fn eval_fn<'h>(
    func: impl Fn(&[f32]) -> Option<f32>,
    tokens: &mut PeekMatch<'h>,
    vars: &mut impl MapIndexable
) -> Option<f32> {
    tokens.next()?; // pop function name

    if let Some(t) = tokens.peek() && t.as_str() == "(" {
        tokens.next()?;

        let mut args = vec![
            eval_expr(tokens, vars)?
        ];

        while let Some(t) = tokens.peek() && t.as_str() == "," {
            tokens.next()?;
            args.push(eval_expr(tokens, vars)?)
        }

        if let Some(t) = tokens.next() && t.as_str() == ")" {
            func(&args)
        } else {
            None
        }

    } else {
        None
    }

}

