import os
import subprocess

tests = [
    # Booleans
    ("bool_true", "true", "true", False),
    ("bool_false", "false", "false", False),
    # Comparisons
    ("cmp_lt", "(< 3 5)", "true", False),
    ("cmp_gt", "(> 3 5)", "false", False),
    ("cmp_le", "(<= 5 5)", "true", False),
    ("cmp_ge", "(>= 5 5)", "true", False),
    ("cmp_eq1", "(= 5 5)", "true", False),
    ("cmp_eq2", "(= 5 6)", "false", False),
    ("cmp_eq3", "(= true true)", "true", False),
    ("cmp_eq4", "(= true false)", "false", False),
    ("cmp_err1", "(= true 5)", "invalid argument", True),
    # Type errors
    ("type_err1", "(+ 5 true)", "invalid argument", True),
    ("type_err2", "(< false 5)", "invalid argument", True),
    ("type_err3", "(add1 false)", "invalid argument", True),
    # Type checks
    ("isnum1", "(isnum 5)", "true", False),
    ("isnum2", "(isnum true)", "false", False),
    ("isbool1", "(isbool false)", "true", False),
    ("isbool2", "(isbool 0)", "false", False),
    # Conditionals
    ("if_true", "(if true 5 10)", "5", False),
    ("if_false", "(if false 5 10)", "10", False),
    ("if_nested", "(if (= 5 5) (if (< 3 4) 1 2) 3)", "1", False),
    # Loops and breaks
    ("loop_break", "(let ((x 0)) (loop (if (= x 10) (break x) (set! x (+ x 1)))))", "10", False),
    ("loop_nested", "(let ((x 0) (y 0)) (loop (if (= x 3) (break y) (block (set! x (+ x 1)) (set! y (+ y 2))))))", "6", False),
    # Mutation
    ("set_basic", "(let ((x 5)) (block (set! x 10) x))", "10", False),
    ("set_type_change", "(let ((x 5)) (block (set! x true) x))", "true", False),
    # Mixed numeric and boolean
    ("mixed1", "(let ((b true)) (if b (+ 2 3) 0))", "5", False),
    ("mixed2", "(let ((n 5)) (if (> n 0) true false))", "true", False),
    # Input
    ("input_test", "input", "10", False, "10"),
    ("input_bool", "input", "true", False, "true"),
    # Functions
    ("fun_simple", "(fun (id x) x) (id 5)", "5", False),
    ("fun_mult", "(fun (add2 x y) (+ x y)) (add2 4 5)", "9", False),
    ("fun_0arg", "(fun (ret5) 5) (ret5)", "5", False),
    ("fun_recursive", "(fun (fact n) (if (= n 1) 1 (* n (fact (- n 1))))) (fact 5)", "120", False),
    ("fun_mutual_rec", "(fun (even n) (if (= n 0) true (odd (- n 1)))) (fun (odd n) (if (= n 0) false (even (- n 1)))) (even 4)", "true", False),
    ("fun_nested_call", "(fun (f x) (+ x 1)) (fun (g x) (f (f x))) (g 10)", "12", False),
    ("fun_locals", "(fun (compute x) (let ((y (* x 2)) (z (+ y 1))) (- z x))) (compute 10)", "11", False),
    ("fun_mixed", "(fun (f x) (let ((y 2)) (+ x y))) (f 3)", "5", False),
    ("fun_print", "(fun (f x) (print x)) (f 42)", "42\n42", False),
    ("fun_print_bool", "(fun (f x) (print x)) (f true)", "true\ntrue", False),
    ("fun_args_many", "(fun (f a b c d e) (+ a (+ b (+ c (+ d e))))) (f 1 2 3 4 5)", "15", False),
    ("fun_tail", "(fun (t x) (if (= x 0) 99 (t (- x 1)))) (t 10)", "99", False),
    ("fun_err_undef", "(f 5)", "not defined", True),
    ("fun_err_arity", "(fun (f x) x) (f 1 2)", "expected 1 arguments, got 2", True),
    ("fun_shadow", "(fun (f x) (let ((x (+ x 1))) x)) (f 5)", "6", False),
    ("fun_call_in_let", "(fun (f x) (+ x 1)) (let ((y (f 10))) (+ y 2))", "13", False),
    ("fun_call_in_op", "(fun (f x) (* x 2)) (+ (f 3) (f 4))", "14", False),
    ("fun_nested_def", "(fun (f x) (let ((y (let ((z 1)) (+ x z)))) y)) (f 5)", "6", False),
    ("fun_err_dup_param", "(fun (f x x) x) (f 1 2)", "Duplicate parameter", True),
    ("fun_err_invalid_name", "(fun (let x) x) (let 5)", "Invalid function name", True),
    ("fun_err_invalid_param", "(fun (f let) let) (f 5)", "Invalid parameter", True),
    ("fun_loop", "(fun (f x) (loop (if (= x 0) (break 1) (set! x (- x 1))))) (f 3)", "1", False),
    ("fun_block", "(fun (f x) (block (set! x (+ x 1)) x)) (f 5)", "6", False),
    ("fun_multi_rec", "(fun (f x y) (if (= x 0) y (f (- x 1) (+ y 2)))) (f 5 0)", "10", False),
    ("fun_set_param", "(fun (f x) (block (set! x 10) x)) (f 5)", "10", False),
    ("fun_isnum", "(fun (f x) (isnum x)) (f 5)", "true", False),
    ("fun_isbool", "(fun (f x) (isbool x)) (f 5)", "false", False),
]

def make_test(name, code, expected, is_error, input_val=None):
    with open(f"test/{name}.snek", "w") as f:
        f.write(code)
    
    # build
    print(f"Running {name}...")
    res = subprocess.run(["make", f"test/{name}.run"], capture_output=True, text=True)
    if res.returncode != 0:
        if is_error and expected.lower() in res.stderr.lower():
            print(f"Pass: {name}")
            return True
        print(f"Fail: {name} failed to compile!")
        print(res.stderr)
        return False
        
    # run
    cmd = [f"./test/{name}.run"]
    if input_val:
        cmd.append(input_val)
        
    res = subprocess.run(cmd, capture_output=True, text=True)
    out = res.stdout.strip()
    err = res.stderr.strip()
    
    if is_error:
        if expected.lower() not in err.lower():
            print(f"Fail: {name} expected error '{expected}', got stderr '{err}' and stdout '{out}'")
            return False
    else:
        if out != expected:
            print(f"Fail: {name} expected '{expected}', got '{out}'")
            return False
            
    print(f"Pass: {name}")
    return True

successes = 0
for t in tests:
    if len(t) == 4:
        name, code, expected, is_error = t
        input_val = None
    else:
        name, code, expected, is_error, input_val = t
        
    if make_test(name, code, expected, is_error, input_val):
        successes += 1

print(f"\\nResult: {successes} / {len(tests)} passed.")
if successes != len(tests):
    exit(1)
