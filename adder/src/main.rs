use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use im::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op1 { Add1, Sub1 }

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op2 { Plus, Minus, Times }

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Number(i32),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
}

fn is_valid_identifier(name: &str) -> bool {
    if name == "let" || name == "add1" || name == "sub1" {
        return false;
    }
    if name.is_empty() { return false; }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() { return false; }
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
            return false;
        }
    }
    true
}

fn parse_bind(s: &Sexp) -> (String, Expr) {
    match s {
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(name)), e] => {
                    if !is_valid_identifier(name) {
                        panic!("Invalid");
                    }
                    (name.clone(), parse_expr(e))
                }
                _ => panic!("Invalid"),
            }
        }
        _ => panic!("Invalid"),
    }
}

fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => match i32::try_from(*n) {
            Ok(val) => Expr::Number(val),
            Err(_) => panic!("Invalid"),
        },
        Sexp::Atom(S(name)) => {
            if !is_valid_identifier(name) {
                panic!("Invalid");
            }
            Expr::Id(name.clone())
        }
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), e] if op == "add1" => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e))),
                [Sexp::Atom(S(op)), e] if op == "sub1" => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e))),
                [Sexp::Atom(S(op)), e1, e2] if op == "+" => Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Sexp::Atom(S(op)), Sexp::List(binds), e] if op == "let" => {
                    if binds.is_empty() {
                        panic!("Invalid");
                    }
                    let mut parsed_binds = Vec::new();
                    for bind in binds {
                        parsed_binds.push(parse_bind(bind));
                    }
                    Expr::Let(parsed_binds, Box::new(parse_expr(e)))
                }
                _ => panic!("Invalid"),
            }
        },
        _ => panic!("Invalid"),
    }
}

fn compile_expr(e: &Expr, env: &HashMap<String, i32>, si: i32) -> String {
    match e {
        Expr::Number(n) => format!("mov rax, {}", *n),
        Expr::Id(name) => {
            match env.get(name) {
                Some(offset) => {
                    let offset_str = if *offset < 0 {
                        format!("- {}", -offset)
                    } else {
                        format!("+ {}", offset)
                    };
                    format!("mov rax, [rsp {}]", offset_str)
                },
                None => panic!("Unbound variable identifier {}", name),
            }
        }
        Expr::UnOp(op, subexpr) => {
            let compiled_subexpr = compile_expr(subexpr, env, si);
            match op {
                Op1::Add1 => format!("{}\nadd rax, 1", compiled_subexpr),
                Op1::Sub1 => format!("{}\nsub rax, 1", compiled_subexpr),
            }
        }
        Expr::BinOp(op, e1, e2) => {
            let c1 = compile_expr(e1, env, si);
            let offset = si * 8;
            let save_c1 = format!("mov [rsp - {}], rax", offset);
            let c2 = compile_expr(e2, env, si + 1);
            let compute = match op {
                Op2::Plus => format!("add rax, [rsp - {}]", offset),
                Op2::Minus => format!("mov r8, rax\nmov rax, [rsp - {}]\nsub rax, r8", offset),
                Op2::Times => format!("imul rax, [rsp - {}]", offset),
            };
            format!("{}\n{}\n{}\n{}", c1, save_c1, c2, compute)
        }
        Expr::Let(binds, body) => {
            let mut current_env = env.clone();
            let mut current_si = si;
            let mut instrs = String::new();
            
            let mut seen = std::collections::HashSet::new();
            for (name, val) in binds {
                if !seen.insert(name.clone()) {
                    panic!("Duplicate binding");
                }
                let val_instrs = compile_expr(val, &current_env, current_si);
                let offset = current_si * 8;
                let save = format!("mov [rsp - {}], rax", offset);
                
                if !instrs.is_empty() {
                    instrs.push_str("\n");
                }
                instrs.push_str(&val_instrs);
                instrs.push_str("\n");
                instrs.push_str(&save);
                current_env = current_env.update(name.clone(), -(offset as i32));
                current_si += 1;
            }
            let body_instrs = compile_expr(body, &current_env, current_si);
            if !instrs.is_empty() {
                instrs.push_str("\n");
            }
            instrs.push_str(&body_instrs);
            instrs
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let parsed_sexp = match parse(&in_contents) {
        Ok(s) => s,
        Err(_) => panic!("Invalid"),
    };
    let expr = parse_expr(&parsed_sexp);
    
    let env = HashMap::new();
    let si = 2;
    let result = compile_expr(&expr, &env, si);

    let asm_program = format!(
        "
section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
",
        result
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}