.SILENT:
start-database:
	./scripts/init_db.sh

test:
	cargo test

check:
	cargo check

build-image:
	docker build -t z2p .

run:
	cargo run
