// Golden output of the sample programs, run through the real `speak` binary.
use std::process::Command;

const FIZZBUZZ: &str = "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n";

fn run_sample(locale: &str, sample: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_speak"))
        .env("SPEAK", locale)
        .args(["run", &format!("core/samples/{sample}")])
        .output()
        .expect("the speak binary runs");
    String::from_utf8(out.stdout).expect("output is utf-8")
}

// `speak run` ends by printing the program's last value after the program's own output,
// so only the start of stdout is compared.
#[test]
fn fizzbuzz_english() {
    assert!(run_sample("en", "fizzbuzz.spk").starts_with(FIZZBUZZ));
}

#[test]
fn fizzbuzz_swahili() {
    assert!(run_sample("sw", "fizzbuzz_swa.spk").starts_with(FIZZBUZZ));
}

#[test]
fn forloop_sample() {
    let out = run_sample("en", "forloop.spk");
    assert!(out.starts_with("1\n2\n3\n10 did not print\n3\n6\n"), "{out}");
}
