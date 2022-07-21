run:
	cargo run

.SILENT:
start-postgres:
	./scripts/init_db.sh

.SILENT:
start-redis:
	./scripts/init_redis.sh

test:
	cargo test

check:
	cargo check

build-image:
	docker build -t z2p .
