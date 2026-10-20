build:
	cargo build --release

# RUNS=n overrides runs per benchmark (default 5).
bench: build
	./benches/run.sh
