build:
	docker compose build
db:
	docker compose up

dev:
	/Users/kenkaminakaya/.asdf/installs/rust/1.72.1/bin/sqlx db create
	/Users/kenkaminakaya/.asdf/installs/rust/1.72.1/bin/sqlx migrate run
	cargo watch -x run

test:
	cargo test

# standalone test
test-s:
	cargo test --no-default-features