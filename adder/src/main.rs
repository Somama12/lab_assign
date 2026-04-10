use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use im::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op1 {
    Add1, Sub1, Negate, IsNum, IsBool, Print
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Program {
    defns: Vec<Definition>,
    main: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Definition {
    name: String,
    params: Vec<String>,
    body: Expr,
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
    Call(String, Vec<Expr>),
}

fn is_valid_identifier(name: &str) -> bool {
    let reserved = [
        "let", "add1", "sub1", "negate", "isnum", "isbool", "print", "true", "false", "input", "if", "block", "loop", "break", "set!", "fun"
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

fn parse_program(s: &Sexp) -> Program {
    match s {
        Sexp::List(items) => {
            let mut defns = vec![];
            let mut main_expr = None;
            
            for item in items {
                if let Some(defn) = try_parse_defn(item) {
                    defns.push(defn);
                } else if main_expr.is_none() {
                    main_expr = Some(parse_expr(item));
                } else {
                    panic!("Multiple main expressions or expression before definitions");
                }
            }
            
            Program {
                defns,
                main: main_expr.expect("No main expression"),
            }
        }
        _ => panic!("Invalid program"),
    }
}

fn try_parse_defn(s: &Sexp) -> Option<Definition> {
    match s {
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(fun)), Sexp::List(signature), body] if fun == "fun" => {
                match &signature[..] {
                    [Sexp::Atom(S(name)), params @ ..] => {
                        if !is_valid_identifier(name) {
                            panic!("Invalid function name: {}", name);
                        }
                        let mut param_names = Vec::new();
                        for p in params {
                            match p {
                                Sexp::Atom(S(p_name)) => {
                                    if !is_valid_identifier(p_name) {
                                        panic!("Invalid parameter name: {}", p_name);
                                    }
                                    if param_names.contains(p_name) {
                                        panic!("Duplicate parameter name: {}", p_name);
                                    }
                                    param_names.push(p_name.clone());
                                }
                                _ => panic!("Invalid parameter format"),
                            }
                        }
                        
                        Some(Definition {
                            name: name.clone(),
                            params: param_names,
                            body: parse_expr(body),
                        })
                    }
                    _ => panic!("Invalid function signature"),
                }
            }
            _ => None,
        },
        _ => None,
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
                    "add1" | "sub1" | "negate" | "isnum" | "isbool" | "print" => {
                        if vec.len() != 2 { panic!("Invalid unary op arity"); }
                        let subexpr = Box::new(parse_expr(&vec[1]));
                        let uop = match op.as_str() {
                            "add1" => Op1::Add1,
                            "sub1" => Op1::Sub1,
                            "negate" => Op1::Negate,
                            "isnum" => Op1::IsNum,
                            "isbool" => Op1::IsBool,
                            "print" => Op1::Print,
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
                    _ => {
                        if is_valid_identifier(op) {
                            let args = vec[1..].iter().map(parse_expr).collect();
                            Expr::Call(op.clone(), args)
                        } else {
                            panic!("Invalid operation: {}", op)
                        }
                    }
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
    funs: &HashMap<String, usize>,
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
            if *b { "mov rax, 3".to_string() } else { "mov rax, 1".to_string() }
        }
        Expr::Input => "mov rax, r15".to_string(),
        Expr::Id(name) => {
            match env.get(name) {
                Some(offset) => {
                    let offset_str = if *offset > 0 { format!("+ {}", offset) } else { format!("- {}", -offset) };
                    format!("mov rax, [rbp {}]", offset_str)
                },
                None => panic!("Unbound variable identifier {}", name),
            }
        }
        Expr::UnOp(op, subexpr) => {
            let compiled_subexpr = compile_expr(subexpr, env, funs, si, l, break_target);
            let check_num = format!("mov rbx, rax\nand rbx, 1\ncmp rbx, 0\njne invalid_argument_error");
            match op {
                Op1::Add1 => format!("{}\n{}\nadd rax, 2\njo overflow_error", compiled_subexpr, check_num),
                Op1::Sub1 => format!("{}\n{}\nsub rax, 2\njo overflow_error", compiled_subexpr, check_num),
                Op1::Negate => format!("{}\n{}\nneg rax\njo overflow_error", compiled_subexpr, check_num),
                Op1::IsNum => format!("{}\nand rax, 1\nshl rax, 1\nxor rax, 3", compiled_subexpr),
                Op1::IsBool => format!("{}\nand rax, 1\nshl rax, 1\nadd rax, 1", compiled_subexpr),
                Op1::Print => format!("{}\nmov rdi, rax\ncall snek_print", compiled_subexpr),
            }
        }
        Expr::BinOp(op, e1, e2) => {
            let c1 = compile_expr(e1, env, funs, si, l, break_target);
            let offset = si * 8;
            let save_c1 = format!("mov [rbp - {}], rax", offset);
            let c2 = compile_expr(e2, env, funs, si + 1, l, break_target);
            let check_nums = format!("mov rbx, rax\nor rbx, [rbp - {}]\ntest rbx, 1\njnz invalid_argument_error", offset);
            let compute = match op {
                Op2::Plus => format!("{}\nadd rax, [rbp - {}]\njo overflow_error", check_nums, offset),
                Op2::Minus => format!("{}\nmov r8, rax\nmov rax, [rbp - {}]\nsub rax, r8\njo overflow_error", check_nums, offset),
                Op2::Times => format!("{}\nmov r8, rax\nmov rax, [rbp - {}]\nsar rax, 1\nimul rax, r8\njo overflow_error", check_nums, offset),
                Op2::Equal => {
                    let label_true = new_label(l, "cmp_true");
                    let label_end = new_label(l, "cmp_end");
                    format!("mov r8, rax\nmov r9, [rbp - {}]\nmov r10, r8\nxor r10, r9\ntest r10, 1\njnz invalid_argument_error\ncmp r9, r8\nje {}\nmov rax, 1\njmp {}\n{}:\nmov rax, 3\n{}:",
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
                    format!("{}\nmov r8, rax\nmov rax, [rbp - {}]\ncmp rax, r8\n{} {}\nmov rax, 1\njmp {}\n{}:\nmov rax, 3\n{}:",
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
                if seen.contains(name) { panic!("Duplicate binding"); }
                seen.push(name.clone());
                let val_instrs = compile_expr(val, &current_env, funs, current_si, l, break_target);
                let offset = current_si * 8;
                let save = format!("mov [rbp - {}], rax", offset);
                if !instrs.is_empty() { instrs.push_str("\n"); }
                instrs.push_str(&val_instrs);
                instrs.push_str("\n");
                instrs.push_str(&save);
                current_env = current_env.update(name.clone(), -(offset as i32));
                current_si += 1;
            }
            let body_instrs = compile_expr(body, &current_env, funs, current_si, l, break_target);
            if !instrs.is_empty() { instrs.push_str("\n"); }
            instrs.push_str(&body_instrs);
            instrs
        }
        Expr::If(cond, thn, els) => {
            let else_label = new_label(l, "if_else");
            let end_label = new_label(l, "if_end");
            let c_cond = compile_expr(cond, env, funs, si, l, break_target);
            let c_thn = compile_expr(thn, env, funs, si, l, break_target);
            let c_els = compile_expr(els, env, funs, si, l, break_target);
            format!("{}\ncmp rax, 1\nje {}\n{}\njmp {}\n{}:\n{}\n{}:", c_cond, else_label, c_thn, end_label, else_label, c_els, end_label)
        }
        Expr::Block(exprs) => {
            let mut instrs = Vec::new();
            for exp in exprs { instrs.push(compile_expr(exp, env, funs, si, l, break_target)); }
            instrs.join("\n")
        }
        Expr::Loop(body) => {
            let loop_start = new_label(l, "loop_start");
            let loop_end = new_label(l, "loop_end");
            let c_body = compile_expr(body, env, funs, si, l, &Some(loop_end.clone()));
            format!("{}:\n{}\njmp {}\n{}:", loop_start, c_body, loop_start, loop_end)
        }
        Expr::Break(exp) => {
            match break_target {
                Some(end_lbl) => {
                    let c_exp = compile_expr(exp, env, funs, si, l, break_target);
                    format!("{}\njmp {}", c_exp, end_lbl)
                }
                None => panic!("break outside of loop"),
            }
        }
        Expr::Set(name, exp) => {
            let c_exp = compile_expr(exp, env, funs, si, l, break_target);
            match env.get(name) {
                Some(offset) => {
                    let offset_str = if *offset > 0 { format!("+ {}", offset) } else { format!("- {}", -offset) };
                    format!("{}\nmov [rbp {}], rax", c_exp, offset_str)
                },
                None => panic!("Unbound variable identifier {}", name),
            }
        }
        Expr::Call(name, args) => {
            let expected_arity = match funs.get(name) {
                Some(arity) => *arity,
                None => panic!("Function {} is not defined", name),
            };
            if args.len() != expected_arity {
                panic!("Function {} expected {} arguments, got {}", name, expected_arity, args.len());
            }
            let mut instrs = vec![];
            let mut current_si = si;
            for arg in args.iter() {
                instrs.push(compile_expr(arg, env, funs, current_si, l, break_target));
                instrs.push(format!("mov [rbp - {}], rax", current_si * 8));
                current_si += 1;
            }
            let alignment = if args.len() % 2 != 0 { 8 } else { 0 };
            if alignment != 0 { instrs.push(format!("sub rsp, {}", alignment)); }
            for i in (0..args.len()).rev() {
                instrs.push(format!("mov rax, [rbp - {}]", (si + i as i32) * 8));
                instrs.push("push rax".to_string());
            }
            instrs.push(format!("call fun_{}", name));
            let total_cleanup = args.len() * 8 + alignment;
            if total_cleanup != 0 { instrs.push(format!("add rsp, {}", total_cleanup)); }
            instrs.join("\n")
        }
    }
}

fn max_locals(e: &Expr) -> i32 {
    match e {
        Expr::Num(_) | Expr::Bool(_) | Expr::Input | Expr::Id(_) => 0,
        Expr::UnOp(_, e1) => max_locals(e1),
        Expr::BinOp(_, e1, e2) => std::cmp::max(max_locals(e1), max_locals(e2) + 1),
        Expr::Let(binds, body) => {
            let mut m = 0;
            for (i, (_, val)) in binds.iter().enumerate() {
                m = std::cmp::max(m, max_locals(val) + i as i32);
            }
            std::cmp::max(m, max_locals(body) + binds.len() as i32)
        }
        Expr::If(e1, e2, e3) => {
            std::cmp::max(max_locals(e1), std::cmp::max(max_locals(e2), max_locals(e3)))
        }
        Expr::Block(es) => es.iter().map(max_locals).max().unwrap_or(0),
        Expr::Loop(e1) => max_locals(e1),
        Expr::Break(e1) => max_locals(e1),
        Expr::Set(_, e1) => max_locals(e1),
        Expr::Call(_, args) => {
            let mut m = 0;
            for (i, arg) in args.iter().enumerate() {
                m = std::cmp::max(m, max_locals(arg) + i as i32);
            }
            m
        }
    }
}

fn compile_defn(defn: &Definition, funs: &HashMap<String, usize>, l: &mut i32) -> String {
    let mut instrs = vec![];
    instrs.push(format!("fun_{}:", defn.name));
    instrs.push("push rbp".to_string());
    instrs.push("mov rbp, rsp".to_string());
    
    let mut env = HashMap::new();
    for (i, param) in defn.params.iter().enumerate() {
        env.insert(param.clone(), 16 + i as i32 * 8);
    }
    
    let mut n = max_locals(&defn.body) + 1;
    if n % 2 != 0 { n += 1; }
    if n > 0 {
        instrs.push(format!("sub rsp, {}", n * 8));
    }
    
    let body_instrs = compile_expr(&defn.body, &env, funs, 1, l, &None);
    instrs.push(body_instrs);
    
    instrs.push("mov rsp, rbp".to_string());
    instrs.push("pop rbp".to_string());
    instrs.push("ret".to_string());
    
    instrs.join("\n  ")
}

fn compile_program(prog: &Program) -> String {
    let mut funs = HashMap::new();
    for defn in &prog.defns {
        if funs.contains_key(&defn.name) {
            panic!("Duplicate function: {}", defn.name);
        }
        funs.insert(defn.name.clone(), defn.params.len());
    }

    let mut label_counter = 0;
    let mut asm = vec![];
    
    for defn in &prog.defns {
        asm.push(compile_defn(defn, &funs, &mut label_counter));
    }
    
    asm.push("our_code_starts_here:".to_string());
    asm.push("mov r15, rdi".to_string());
    asm.push("push rbp".to_string());
    asm.push("mov rbp, rsp".to_string());
    
    let mut n = max_locals(&prog.main) + 1;
    if n % 2 != 0 { n += 1; }
    if n > 0 {
        asm.push(format!("sub rsp, {}", n * 8));
    }
    
    let main_instrs = compile_expr(&prog.main, &HashMap::new(), &funs, 1, &mut label_counter, &None);
    asm.push(main_instrs);
    
    asm.push("mov rsp, rbp".to_string());
    asm.push("pop rbp".to_string());
    asm.push("ret".to_string());
    
    format!("section .text\nglobal our_code_starts_here\nextern snek_error\nextern snek_print\n\n{}\n\ninvalid_argument_error:\n  mov rdi, 1\n  push rbp\n  call snek_error\n\noverflow_error:\n  mov rdi, 2\n  push rbp\n  call snek_error\n", asm.join("\n\n"))
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
    let in_contents = format!("(\n{}\n)", in_contents);

    let parsed_sexp = match parse(&in_contents) {
        Ok(s) => s,
        Err(_) => panic!("Invalid"),
    };
    let prog = parse_program(&parsed_sexp);
    
    let asm_program = compile_program(&prog);

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}