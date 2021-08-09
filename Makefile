init: common

	git submodule update --init --recursive
	./scripts/init.sh
build:
	rm ./common/Cargo.toml
	cargo build --release
push:

	git push -u origin main