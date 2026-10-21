//! Conformance suite: runs each sample in core/samples with its locale and
//! compares stdout and exit status to the checked-in `<sample>.out` golden file
//! (first line `exit: <code>`, then stdout). Regenerate with `UPDATE_GOLDEN=1 cargo test`.
use std::{path::Path, process::Command};

fn check(sample: &str, locale: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("core/samples");
    let out = Command::new(env!("CARGO_BIN_EXE_speak"))
        .args(["run", &format!("{sample}.spk")])
        .env("SPEAK", locale)
        .current_dir(&dir)
        .output()
        .unwrap();
    let actual = format!(
        "exit: {}\n{}",
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout)
    );
    let golden = dir.join(format!("{sample}.out"));
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(&golden, &actual).unwrap();
    }
    let expected = std::fs::read_to_string(&golden).expect("missing golden file");
    assert_eq!(actual, expected, "{sample} ({locale})");
}

macro_rules! conformance {
    ($($name:ident $locale:literal $(ignore $why:literal)?),* $(,)?) => {
        $(
            $(#[ignore = $why])?
            #[test]
            fn $name() { check(stringify!($name), $locale); }
        )*
    };
}

// `ignore` marks a known failure: the golden is kept as the current output, run with
// `cargo test -- --ignored` to see it. Fix the bug, regenerate the golden, drop the `ignore`.
conformance! {
    collatz "en" ignore "known bug: stack overflow (SIGABRT) on recursive collatz 1_000",
    collatz_swa "sw" ignore "known bug: stack overflow (SIGABRT) on recursive collatz 1_000",
    error "en", fib "en", fib_swa "sw",
    blocks "en", fizzbuzz "en", fizzbuzz_swa "sw", forloop "en", habari_dunia "sw",
    hello_world "en",
    object "en" ignore "known bug: object fields print in random HashMap order",
}
