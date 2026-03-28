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
]

def make_test(name, code, expected, is_error, input_val=None):
    with open(f"test/{name}.snek", "w") as f:
        f.write(code)
    
    # build
    print(f"Running {name}...")
    res = subprocess.run(["make", f"test/{name}.run"], capture_output=True, text=True)
    if res.returncode != 0:
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
        if expected not in err:
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
