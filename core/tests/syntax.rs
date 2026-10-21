// Tests for the SPK-114 syntax: if/else blocks, the three `for` forms, ranges, and/or/not
// and school-maths precedence. Programs run through `Context::exec` and the value of the
// last top-level expression is compared as a string.
use speak_core::runtime::Context;
use std::{io::BufReader, sync::Mutex};

// the locale is process-global, so tests that exec programs must not interleave
static LOCALE: Mutex<()> = Mutex::new(());

fn run_in(locale: &str, src: &str) -> Result<String, String> {
    let _guard = LOCALE.lock().unwrap_or_else(|e| e.into_inner());
    Context::new(&false)
        .exec(locale, BufReader::new(src.as_bytes()))
        .map(|(val, ..)| val.string())
        .map_err(|err| err.message)
}

fn run(src: &str) -> String {
    run_in("en", src).unwrap_or_else(|e| panic!("{e}\n--- program ---\n{src}"))
}

#[test]
fn if_else_if_else_blocks() {
    let grade = |n: u32| {
        run(&format!(
            "grade: n number -> string
    if n > 89
        return \"A\"
    else if n > 79
        return \"B\"
    else
        return \"C\"
grade {n}"
        ))
    };
    assert_eq!((grade(95), grade(85), grade(10)), ("A".into(), "B".into(), "C".into()));

    // a block holds several statements, and an `if` without `else` yields ()
    assert_eq!(run("x is 1\nif x = 1\n    x is x + 1\n    x is x * 10\nx"), "20");
    assert_eq!(run("if 1 > 2\n    5"), "");

    // blocks nest, and `else` belongs to the `if` it lines up with
    let src = "x is 0
if true
    if false
        x is 1
    else
        x is 2
else
    x is 3
x";
    assert_eq!(run(src), "2");

    // the value of an `if` is the value of the branch taken
    assert_eq!(run("if 2 > 1\n    \"yes\"\nelse\n    \"no\""), "yes");
}

#[test]
fn old_if_syntax_is_gone() {
    assert!(run_in("en", "if true ? 1 ! 2").is_err());
    assert!(run_in("en", "x is true & false").is_err());
}

#[test]
fn for_in_with_break_and_continue() {
    let src = "total is 0
for x in [1, 2, 3, 4, 5, 6]
    if x = 2
        continue
    if x = 5
        break
    total is total + x
total";
    assert_eq!(run(src), "8"); // 1 + 3 + 4
}

#[test]
fn for_with_condition() {
    assert_eq!(run("n is 0\nfor n < 5\n    n is n + 1\nn"), "5");

    let src = "n is 0
odds is 0
for n < 10
    n is n + 1
    if n % 2 = 0
        continue
    if n > 7
        break
    odds is odds + 1
odds";
    assert_eq!(run(src), "4"); // 1 3 5 7

    // a condition that is not a bool is an error
    assert!(run_in("en", "for 1\n    break").is_err());
}

#[test]
fn bare_for_runs_until_break() {
    let src = "n is 0
for
    n is n + 1
    if n = 3
        continue
    if n > 5
        break
n";
    assert_eq!(run(src), "6");
}

#[test]
fn loop_variable_does_not_leak() {
    assert!(run_in("en", "for i in 1..3\n    i\ni").is_err());
}

#[test]
fn ranges_include_both_ends() {
    assert_eq!(run("n is 0\nfor i in 1..15\n    n is n + 1\nn"), "15");
    assert_eq!(run("s is 0\nfor i in 1..4\n    s is s + i\ns"), "10");
    assert_eq!(run("s is 0\nfor i in 3..3\n    s is s + i\ns"), "3");
    assert_eq!(run("s is 0\nfor i in 5..1\n    s is s + 1\ns"), "0");
    assert_eq!(run("lo is 2\nhi is 4\ns is 0\nfor i in lo..hi + 1\n    s is s + i\ns"), "14");
    assert_eq!(run("s is 0\nfor i in ~2..2\n    s is s + i\ns"), "0");

    // array slices keep their exclusive end
    assert_eq!(run("xs is [1, 2, 3, 4]\nlen xs[1..3]"), "2");

    assert!(run_in("en", "for i in 1..2.5\n    i").is_err());
}

#[test]
fn and_or_not() {
    assert_eq!(run("true and false"), "false");
    assert_eq!(run("true and true"), "true");
    assert_eq!(run("false or true"), "true");
    assert_eq!(run("false or false"), "false");
    assert_eq!(run("not false"), "true");
    assert_eq!(run("not 1 = 2"), "true"); // not (1 = 2)
    assert_eq!(run("not true and false"), "false"); // (not true) and false
    assert_eq!(run("true or false and false"), "true"); // and binds tighter than or
    assert_eq!(run("1 < 2 and 2 < 3"), "true");
    assert_eq!(run("ok is 1 < 2 and not 3 < 2\nok"), "true");
    assert!(run_in("en", "1 and true").is_err());
    assert!(run_in("en", "not 1").is_err());
}

#[test]
fn and_or_short_circuit() {
    // the right side would be a division by zero error if it were evaluated
    assert_eq!(run("false and 1 / 0 = 1"), "false");
    assert_eq!(run("true or 1 / 0 = 1"), "true");
    assert!(run_in("en", "true and 1 / 0 = 1").is_err());
    assert!(run_in("en", "false or 1 / 0 = 1").is_err());
}

#[test]
fn school_maths_precedence() {
    assert_eq!(run("2 + 3 * 4 % 5"), "4"); // 2 + ((3 * 4) % 5)
    assert_eq!(run("2 * 7 % 4"), "2"); // (2 * 7) % 4
    assert_eq!(run("7 % 4 * 2"), "6"); // (7 % 4) * 2
    assert_eq!(run("10 - 4 / 2 * 3"), "4"); // 10 - ((4 / 2) * 3)
    assert_eq!(run("10 - 2 - 3"), "5");
    assert_eq!(run("20 / 5 / 2"), "2");
    assert_eq!(run("1 + 2 * 3 = 7"), "true");
    assert_eq!(run("2 + 3 < 4 + 2"), "true");
    assert_eq!(run("x is 2 + 3 * 4\nx"), "14");
}

#[test]
fn blocks_run_every_statement() {
    // a bare expression in a block must not end the block
    let src = "log is \">\"
x is 1
if x = 1
    x + 1
    log is log + \"a\"
for i in 1..3
    len \"ab\"
    log is log + (sprint i)
log";
    assert_eq!(run(src), ">a123");

    // same inside a function with no return: every statement runs, result is ()
    let src = "log is \">\"
f: n number -> number
    n + 1
    log is log + \"x\"
    n + 2
f 1
log";
    assert_eq!(run(src), ">x");
    assert_eq!(run("f: n number -> number\n    n + 1\nf 1"), "");
}

#[test]
fn return_from_nested_blocks() {
    let src = "first_big: xs []number -> number
    for x in xs
        if x > 10
            return x
    return 0
first_big ([3, 12, 40])";
    assert_eq!(run(src), "12");

    // falls through to the final return when nothing matches
    let src = "first_big: xs []number -> number
    for x in xs
        if x > 100
            return x
    return 0
first_big ([3, 12, 40])";
    assert_eq!(run(src), "0");

    // return from two loops and two ifs deep
    let src = "find: n number -> number
    for i in 1..5
        for j in 1..5
            if i * j = n
                if i < j
                    return i * 10 + j
    return 0
find 12";
    assert_eq!(run(src), "34");

    // statements after a returning branch do not run
    let src = "log is \">\"
f: n number -> number
    if n > 0
        return 1
    log is log + \"neg\"
    return 2
f 5
f (~5)
log";
    assert_eq!(run(src), ">neg");

    let src = "sum_to: n number -> number
    s is 0
    for i in 1..n
        s is s + i
    return s
sum_to 10";
    assert_eq!(run(src), "55");

    // a bare return gives ()
    assert_eq!(run("f: n number -> number\n    return\nf 1"), "");
}

#[test]
fn return_checks_declared_type() {
    assert!(run_in("en", "f: n number -> string\n    return n\nf 1").is_err());
    assert_eq!(run("f: n number -> []number\n    return []\nlen (f 1)"), "0");
}

#[test]
fn return_outside_function_is_an_error() {
    assert!(run_in("en", "return 1").is_err());
    assert!(run_in("en", "for i in 1..2\n    return 1").is_err());
}

#[test]
fn break_and_continue_through_nested_if() {
    let src = "log is \">\"
for i in 1..6
    if i > 1
        if i = 2
            continue
        else
            if i = 5
                break
    log is log + (sprint i)
log";
    assert_eq!(run(src), ">134");

    let src = "log is \">\"
for i in 1..6
    if i % 2 = 0
        continue
    if i > 4
        break
    log is log + (sprint i)
log";
    assert_eq!(run(src), ">13");

    // break and continue are not allowed to cross a function call
    assert!(run_in("en", "f: n number -> number\n    break\nfor i in 1..2\n    f 1").is_err());
}

#[test]
fn swahili_keywords() {
    let src = "j ni 0
kwa i katika 1..4
    kama i % 2 = 0
        endelea
    sivyo kama i = 3
        vunja
    sivyo
        j ni i
j";
    // j is only set when i = 1: 2 continues, 3 breaks
    assert_eq!(run_in("sw", src).unwrap(), "1");
    assert_eq!(run_in("sw", "si ongo na (ongo au kweli)").unwrap(), "kweli");
    assert_eq!(run_in("sw", "n ni 0\nkwa n < 3\n    n ni n + 1\nn").unwrap(), "3");
}

#[test]
fn fizzbuzz_values_in_both_locales() {
    let en = "out is \">\"
for i in 1..15
    if i % 15 = 0
        out is out + \"FizzBuzz,\"
    else if i % 3 = 0
        out is out + \"Fizz,\"
    else if i % 5 = 0
        out is out + \"Buzz,\"
    else
        out is out + (sprint i) + \",\"
out";
    let sw = "out ni \">\"
kwa i katika 1..15
    kama i % 15 = 0
        out ni out + \"FizzBuzz,\"
    sivyo kama i % 3 = 0
        out ni out + \"Fizz,\"
    sivyo kama i % 5 = 0
        out ni out + \"Buzz,\"
    sivyo
        out ni out + (mlolongo_andika i) + \",\"
out";
    let want = ">1,2,Fizz,4,Buzz,Fizz,7,8,Fizz,Buzz,11,Fizz,13,14,FizzBuzz,";
    assert_eq!(run(en), want);
    assert_eq!(run_in("sw", sw).unwrap(), want);
}
