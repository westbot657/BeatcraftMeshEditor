use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::Peekable;

use indexmap::IndexMap;
use regex::Matches;

use crate::DB_MATH;

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

pub fn eval<M>(expr: &str, var_table: &mut M) -> Option<f32>
where
    M: MapIndexable + Debug
{
    tracing::debug!(target: DB_MATH, ?var_table, "Evaluating expression: {expr}");
    eval_inner(expr, var_table, false)
}

pub fn eval_inner(expr: &str, var_table: &mut impl MapIndexable, is_degrees: bool) -> Option<f32> {
    let splitter = regex::Regex::new(r#"((\d*\.\d+|\d+\.\d*|\d+|\w+)|[/()*+\-%,])"#).unwrap();

    let mut tokens = splitter.find_iter(expr).peekable();

    tracing::debug!(target: DB_MATH, ?tokens, "Evaluating expression from tokens");
    let res = eval_expr(&mut tokens, var_table, is_degrees)?;
    if tokens.next().is_some() || res.is_nan() {
        None
    } else {
        Some(res)
    }
}

type PeekMatch<'a> = Peekable<Matches<'a, 'a>>;

fn eval_expr(
    tokens: &mut PeekMatch<'_>,
    vars: &mut impl MapIndexable,
    is_degrees: bool,
) -> Option<f32> {
    let span = tracing::debug_span!("eval-expr");
    let _guard = span.enter();
    let mut left = eval_term(tokens, vars, is_degrees)?;
    while let Some(token) = tokens.peek() {
        match token.as_str() {
            "+" => {
                left += {
                    tokens.next();
                    tracing::debug!(target: DB_MATH, "ADD");
                    eval_term(tokens, vars, is_degrees)?
                }
            }
            "-" => {
                left -= {
                    tokens.next();
                    tracing::debug!(target: DB_MATH, "SUBTRACT");
                    eval_term(tokens, vars, is_degrees)?
                }
            }
            _ => break,
        }
    }
    tracing::debug!(target: DB_MATH, "Result: {}", left);
    Some(left)
}

fn eval_term(
    tokens: &mut PeekMatch<'_>,
    vars: &mut impl MapIndexable,
    is_degrees: bool,
) -> Option<f32> {
    let span = tracing::debug_span!("eval-term");
    let _guard = span.enter();
    let mut left = eval_factor(tokens, vars, is_degrees)?;
    while let Some(token) = tokens.peek() {
        match token.as_str() {
            "*" => {
                left *= {
                    tokens.next();
                    tracing::debug!(target: DB_MATH, "MULTIPLY");
                    eval_factor(tokens, vars, is_degrees)?
                }
            }
            "/" => {
                left /= {
                    tokens.next();
                    tracing::debug!(target: DB_MATH, "DIVIDE");
                    eval_factor(tokens, vars, is_degrees)?
                }
            }
            "%" => {
                left %= {
                    tokens.next();
                    tracing::debug!(target: DB_MATH, "MODULOUS");
                    eval_factor(tokens, vars, is_degrees)?
                }
            }
            _ => break,
        }
    }
    tracing::debug!(target: DB_MATH, "Result: {}", left);
    Some(left)
}

fn eval_factor(
    tokens: &mut PeekMatch<'_>,
    vars: &mut impl MapIndexable,
    is_degrees: bool,
) -> Option<f32> {
    let span = tracing::debug_span!("eval-factor");
    let _guard = span.enter();
    if let Some(token) = tokens.peek() {
        let tk = token.as_str();
        if let Ok(val) = tk.parse::<f32>() {
            tokens.next();
            tracing::debug!(target: DB_MATH, "Value: {}", val);
            return Some(val);
        }

        match tk {
            "-" => {
                tokens.next()?;
                let span = tracing::debug_span!("NEGATE");
                let _guard = span.enter();
                Some(-eval_factor(tokens, vars, is_degrees)?)
            }
            "pi" => {
                tokens.next()?;
                tracing::debug!("Constant: pi");
                Some(std::f32::consts::PI)
            }
            "e" => {
                tokens.next()?;
                tracing::debug!("Constant: e");
                Some(std::f32::consts::E)
            }

            "cos" => eval_fn(|args| Some(args.first()?.cos()), tokens, vars, is_degrees),
            "sin" => eval_fn(|args| Some(args.first()?.sin()), tokens, vars, is_degrees),
            "tan" => eval_fn(|args| Some(args.first()?.tan()), tokens, vars, is_degrees),
            "sqrt" => eval_fn(|args| Some(args.first()?.sqrt()), tokens, vars, is_degrees),
            "pow" => eval_fn(
                |args| Some(args.first()?.powf(*args.get(1)?)),
                tokens,
                vars,
                is_degrees,
            ),
            "sign" => eval_fn(
                |args| Some(args.first()?.signum()),
                tokens,
                vars,
                is_degrees,
            ),
            "abs" => eval_fn(|args| Some(args.first()?.abs()), tokens, vars, is_degrees),
            "floor" => eval_fn(|args| Some(args.first()?.floor()), tokens, vars, is_degrees),
            "ceil" => eval_fn(|args| Some(args.first()?.ceil()), tokens, vars, is_degrees),
            "ln" => eval_fn(|args| Some(args.first()?.ln()), tokens, vars, is_degrees),
            "rads" => eval_fn(
                |args| Some(args.first()?.to_radians()),
                tokens,
                vars,
                is_degrees,
            ),
            "degs" => eval_fn(
                |args| Some(args.first()?.to_degrees()),
                tokens,
                vars,
                is_degrees,
            ),

            "(" => {
                let _ = tokens.next()?;

                let exp = eval_expr(tokens, vars, is_degrees)?;

                if let Some(t) = tokens.next()
                    && t.as_str() == ")"
                {
                    Some(exp)
                } else {
                    None
                }
            }

            _ => {
                let token = tokens.next()?;
                let tk = token.as_str().to_string();

                if let Some(next) = tokens.peek()
                    && next.as_str() == "="
                {
                    tokens.next();
                    let assign = eval_expr(tokens, vars, is_degrees)?;
                    if let Some(val) = vars.get_mut(&tk) {
                        *val = assign;
                    } else {
                        vars.insert(tk.clone(), assign);
                    }
                }
                vars.get(&tk).copied()
            }
        }
    } else {
        None
    }
}

fn eval_fn<'h>(
    func: impl Fn(&[f32]) -> Option<f32>,
    tokens: &mut PeekMatch<'h>,
    vars: &mut impl MapIndexable,
    is_degrees: bool,
) -> Option<f32> {
    let name = tokens.next()?;
    let name = name.as_str();
    let span = tracing::debug_span!("eval-fn");
    let _guard = span.enter();
    tracing::debug!(target: DB_MATH, "evaluating function '{}'", name);

    if let Some(t) = tokens.peek()
        && t.as_str() == "("
    {
        tokens.next()?;

        let mut args = vec![eval_expr(tokens, vars, is_degrees)?];

        while let Some(t) = tokens.peek()
            && t.as_str() == ","
        {
            tokens.next()?;
            args.push(eval_expr(tokens, vars, is_degrees)?)
        }

        if is_degrees {
            args = args.into_iter().map(|f| f.to_radians()).collect();
        }

        if let Some(t) = tokens.next()
            && t.as_str() == ")"
        {
            func(&args)
        } else {
            None
        }
    } else {
        None
    }
}
