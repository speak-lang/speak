# Conformance tests

`tests/conformance.rs` runs every program in `core/samples/` through the `speak` binary
with the locale it is written for (`SPEAK=en` or `SPEAK=sw`) and compares exit status and
stdout with the golden file next to it, `core/samples/<name>.out`.

Add a test (new tickets add the test before the fix):

1. Put `<name>.spk` in `core/samples/` (use a `_swa` suffix for Swahili variants).
2. Add `<name> "<locale>"` to the `conformance!` list in `tests/conformance.rs`.
3. Write the expected output by hand, or run `UPDATE_GOLDEN=1 cargo test --test conformance`
   to generate `<name>.out` and read it before committing.

A test for an unfixed bug gets `ignore "known bug: ..."` in the list; run those with
`cargo test -- --ignored`. Remove the `ignore` when the bug is fixed.

Run everything with `cargo test` (CI does the same).
