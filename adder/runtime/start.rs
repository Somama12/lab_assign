use std::env;

#[link(name = "our_code")]
extern "C" {
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: i64) -> i64;
}

#[no_mangle]
#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    if errcode == 1 {
        eprintln!("invalid argument");
    } else if errcode == 2 {
        eprintln!("overflow");
    } else {
        eprintln!("an error occurred {}", errcode);
    }
    std::process::exit(1);
}

fn parse_input(input: &str) -> i64 {
    if input == "true" {
        return 3;
    } else if input == "false" {
        return 1;
    }
    match input.parse::<i64>() {
        Ok(n) => n << 1,
        Err(_) => panic!("Invalid input: {}", input),
    }
}

fn print_value(i: i64) {
    if i & 1 == 0 {
        println!("{}", i >> 1);
    } else if i == 3 {
        println!("true");
    } else if i == 1 {
        println!("false");
    } else {
        println!("Unknown value: {}", i);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() == 2 {
        parse_input(&args[1])
    } else {
        1 
    };

    let i: i64 = unsafe {
        our_code_starts_here(input)
    };
    print_value(i);
}