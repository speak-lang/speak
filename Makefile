default:
	cargo build --release

swahili:
	LANG=swahili cargo build --release --features language