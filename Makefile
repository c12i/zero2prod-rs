.SILENT:
start-database:
		./scripts/init_db.sh

test:
		cargo test

build-image:
		docker build -t z2p .

run:
		cargo run
