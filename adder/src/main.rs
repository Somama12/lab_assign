use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use im::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op1 {
    Add1, Sub1, Negate, IsNum, IsBool
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op2 {
    Plus, Minus, Times, Less, Greater, LessEq, GreaterEq, Equal
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Num(i32),
    Bool(bool),
    Input,
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    Loop(Box<Expr>),
    Break(Box<Expr>),
    Set(String, Box<Expr>),
}

fn is_valid_identifier(name: &str) -> bool {
    let reserved = [
        "let", "add1", "sub1", "negate", "isnum", "isbool", "true", "false", "input", "if", "block", "loop", "break", "set!"
    ];
    if reserved.contains(&name) {
        return false;
    }
    if name.is_empty() { return false; }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' { return false; }
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
                        panic!("Invalid identifier in let binding");
                    }
                    (name.clone(), parse_expr(e))
                }
                _ => panic!("Invalid let binding: invalid bind element"),
            }
        }
        _ => panic!("Invalid binding format"),
    }
}

fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => match i32::try_from(*n) {
            Ok(val) => Expr::Num(val),
            Err(_) => panic!("Invalid integer"),
        },
        Sexp::Atom(S(name)) => {
            if name == "true" {
                Expr::Bool(true)
            } else if name == "false" {
                Expr::Bool(false)
            } else if name == "input" {
                Expr::Input
            } else if is_valid_identifier(name) {
                Expr::Id(name.clone())
            } else {
                panic!("Invalid identifier: {}", name);
            }
        }
        Sexp::List(vec) => {
            if vec.is_empty() {
                panic!("Invalid empty list");
            }
            match &vec[0] {
                Sexp::Atom(S(op)) => match op.as_str() {
                    "add1" | "sub1" | "negate" | "isnum" | "isbool" => {
                        if vec.len() != 2 { panic!("Invalid unary op arity"); }
                        let subexpr = Box::new(parse_expr(&vec[1]));
                        let uop = match op.as_str() {
                            "add1" => Op1::Add1,
                            "sub1" => Op1::Sub1,
                            "negate" => Op1::Negate,
                            "isnum" => Op1::IsNum,
                            "isbool" => Op1::IsBool,
                            _ => unreachable!(),
                        };
                        Expr::UnOp(uop, subexpr)
                    }
                    "+" | "-" | "*" | "<" | ">" | "<=" | ">=" | "=" => {
                        if vec.len() != 3 { panic!("Invalid binary op arity"); }
                        let e1 = Box::new(parse_expr(&vec[1]));
                        let e2 = Box::new(parse_expr(&vec[2]));
                        let bop = match op.as_str() {
                            "+" => Op2::Plus, "-" => Op2::Minus, "*" => Op2::Times,
                            "<" => Op2::Less, ">" => Op2::Greater, "<=" => Op2::LessEq,
                            ">=" => Op2::GreaterEq, "=" => Op2::Equal,
                            _ => unreachable!(),
                        };
                        Expr::BinOp(bop, e1, e2)
                    }
                    "let" => {
                        if vec.len() != 3 { panic!("Invalid let arity"); }
                        let binds = match &vec[1] {
                            Sexp::List(b) => {
                                if b.is_empty() { panic!("Invalid let binding empty"); }
                                b.iter().map(parse_bind).collect()
                            }
                            _ => panic!("Invalid let binding format"),
                        };
                        Expr::Let(binds, Box::new(parse_expr(&vec[2])))
                    }
                    "if" => {
                        if vec.len() != 4 { panic!("Invalid if arity"); }
                        Expr::If(
                            Box::new(parse_expr(&vec[1])),
                            Box::new(parse_expr(&vec[2])),
                            Box::new(parse_expr(&vec[3])),
                        )
                    }
                    "block" => {
                        if vec.len() < 2 { panic!("Invalid block arity"); }
                        let exprs = vec[1..].iter().map(parse_expr).collect();
                        Expr::Block(exprs)
                    }
                    "loop" => {
                        if vec.len() != 2 { panic!("Invalid loop arity"); }
                        Expr::Loop(Box::new(parse_expr(&vec[1])))
                    }
                    "break" => {
                        if vec.len() != 2 { panic!("Invalid break arity"); }
                        Expr::Break(Box::new(parse_expr(&vec[1])))
                    }
                    "set!" => {
                        if vec.len() != 3 { panic!("Invalid set arity"); }
                        let name = match &vec[1] {
                            Sexp::Atom(S(n)) if is_valid_identifier(n) => n.clone(),
                            _ => panic!("Invalid set identifier"),
                        };
                        Expr::Set(name, Box::new(parse_expr(&vec[2])))
                    }
                    _ => panic!("Invalid operation: {}", op),
                },
                _ => panic!("Invalid expression format"),
            }
        },
        _ => panic!("Invalid expression"),
    }
}

fn new_label(l: &mut i32, s: &str) -> String {
    let current = *l;
    *l += 1;
    format!("{}_{}", s, current)
}

fn compile_expr(
    e: &Expr,
    env: &HashMap<String, i32>,
    si: i32,
    l: &mut i32,
    break_target: &Option<String>
) -> String {
    match e {
        Expr::Num(n) => {
            if let Some(val) = n.checked_mul(2) {
                format!("mov rax, {}", val)
            } else {
                panic!("Integer overflow during compilation");
            }
        }
        Expr::Bool(b) => {
            if *b {
                "mov rax, 3".to_string()
            } else {
                "mov rax, 1".to_string()
            }
        }
        Expr::Input => {
            "mov rax, r15".to_string()
        }
        Expr::Id(name) => {
            match env.get(name) {
                Some(offset) => {
                    let offset_str = if *offset < 0 { format!("- {}", -offset) } else { format!("+ {}", offset) };
                    format!("mov rax, [rsp {}]", offset_str)
                },
                None => panic!("Unbound variable identifier {}", name),
            }
        }
        Expr::UnOp(op, subexpr) => {
            let compiled_subexpr = compile_expr(subexpr, env, si, l, break_target);
            let check_num = format!("mov rbx, rax\nand rbx, 1\ncmp rbx, 0\njne invalid_argument_error");
            match op {
                Op1::Add1 => format!("{}\n{}\nadd rax, 2\njo overflow_error", compiled_subexpr, check_num),
                Op1::Sub1 => format!("{}\n{}\nsub rax, 2\njo overflow_error", compiled_subexpr, check_num),
                Op1::Negate => format!("{}\n{}\nneg rax\njo overflow_error", compiled_subexpr, check_num),
                Op1::IsNum => format!("{}\nand rax, 1\nshl rax, 1\nxor rax, 3", compiled_subexpr),
                Op1::IsBool => format!("{}\nand rax, 1\nshl rax, 1\nadd rax, 1", compiled_subexpr),
            }
        }
        Expr::BinOp(op, e1, e2) => {
            let c1 = compile_expr(e1, env, si, l, break_target);
            let offset = si * 8;
            let save_c1 = format!("mov [rsp - {}], rax", offset);
            let c2 = compile_expr(e2, env, si + 1, l, break_target);
            
            let check_nums = format!(
                "mov rbx, rax\nor rbx, [rsp - {}]\ntest rbx, 1\njnz invalid_argument_error",
                offset
            );
            
            let compute = match op {
                Op2::Plus => format!("{}\nadd rax, [rsp - {}]\njo overflow_error", check_nums, offset),
                Op2::Minus => format!("{}\nmov r8, rax\nmov rax, [rsp - {}]\nsub rax, r8\njo overflow_error", check_nums, offset),
                Op2::Times => format!("{}\nmov r8, rax\nmov rax, [rsp - {}]\nsar rax, 1\nimul rax, r8\njo overflow_error", check_nums, offset),
                Op2::Equal => {
                    let label_true = new_label(l, "cmp_true");
                    let label_end = new_label(l, "cmp_end");
                    format!("mov r8, rax\nmov r9, [rsp - {}]\nmov r10, r8\nxor r10, r9\ntest r10, 1\njnz invalid_argument_error\ncmp r9, r8\nje {}\nmov rax, 1\njmp {}\n{}:\nmov rax, 3\n{}:",
                        offset, label_true, label_end, label_true, label_end)
                },
                Op2::Less | Op2::Greater | Op2::LessEq | Op2::GreaterEq => {
                    let label_true = new_label(l, "cmp_true");
                    let label_end = new_label(l, "cmp_end");
                    let inst = match op {
                        Op2::Less => "jl", Op2::Greater => "jg",
                        Op2::LessEq => "jle", Op2::GreaterEq => "jge",
                        _ => unreachable!()
                    };
                    format!("{}\nmov r8, rax\nmov rax, [rsp - {}]\ncmp rax, r8\n{} {}\nmov rax, 1\njmp {}\n{}:\nmov rax, 3\n{}:",
                        check_nums, offset, inst, label_true, label_end, label_true, label_end)
                }
            };
            format!("{}\n{}\n{}\n{}", c1, save_c1, c2, compute)
        }
        Expr::Let(binds, body) => {
            let mut current_env = env.clone();
            let mut current_si = si;
            let mut instrs = String::new();
            
            let mut seen: Vec<String> = Vec::new();
            for (name, val) in binds {
                if seen.contains(name) {
                    panic!("Duplicate binding");
                }
                seen.push(name.clone());
                let val_instrs = compile_expr(val, &current_env, current_si, l, break_target);
                let offset = current_si * 8;
                let save = format!("mov [rsp - {}], rax", offset);
                
                if !instrs.is_empty() { instrs.push_str("\n"); }
                instrs.push_str(&val_instrs);
                instrs.push_str("\n");
                instrs.push_str(&save);
                current_env = current_env.update(name.clone(), -(offset as i32));
                current_si += 1;
            }
            let body_instrs = compile_expr(body, &current_env, current_si, l, break_target);
            if !instrs.is_empty() { instrs.push_str("\n"); }
            instrs.push_str(&body_instrs);
            instrs
        }
        Expr::If(cond, thn, els) => {
            let else_label = new_label(l, "if_else");
            let end_label = new_label(l, "if_end");
            let c_cond = compile_expr(cond, env, si, l, break_target);
            let c_thn = compile_expr(thn, env, si, l, break_target);
            let c_els = compile_expr(els, env, si, l, break_target);
            format!(
                "{}\ncmp rax, 1\nje {}\n{}\njmp {}\n{}:\n{}\n{}:",
                c_cond, else_label, c_thn, end_label, else_label, c_els, end_label
            )
        }
        Expr::Block(exprs) => {
            let mut instrs = Vec::new();
            for exp in exprs {
                instrs.push(compile_expr(exp, env, si, l, break_target));
            }
            instrs.join("\n")
        }
        Expr::Loop(body) => {
            let loop_start = new_label(l, "loop_start");
            let loop_end = new_label(l, "loop_end");
            let c_body = compile_expr(body, env, si, l, &Some(loop_end.clone()));
            format!(
                "{}:\n{}\njmp {}\n{}:",
                loop_start, c_body, loop_start, loop_end
            )
        }
        Expr::Break(exp) => {
            match break_target {
                Some(end_lbl) => {
                    let c_exp = compile_expr(exp, env, si, l, break_target);
                    format!("{}\njmp {}", c_exp, end_lbl)
                }
                None => panic!("break outside of loop"),
            }
        }
        Expr::Set(name, exp) => {
            let c_exp = compile_expr(exp, env, si, l, break_target);
            match env.get(name) {
                Some(offset) => {
                    let offset_str = if *offset < 0 { format!("- {}", -offset) } else { format!("+ {}", offset) };
                    format!("{}\nmov [rsp {}], rax", c_exp, offset_str)
                },
                None => panic!("Unbound variable identifier {}", name),
            }
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
    let mut label_counter = 0;
    
    let result = compile_expr(&expr, &env, si, &mut label_counter, &None);

    let asm_program = format!(
        "
section .text
global our_code_starts_here
extern snek_error

our_code_starts_here:
  mov r15, rdi
  {}
  ret

invalid_argument_error:
  mov rdi, 1
  push rbp
  call snek_error

overflow_error:
  mov rdi, 2
  push rbp
  call snek_error
",
        result
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}