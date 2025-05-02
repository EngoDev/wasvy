build-protocol:
	cargo component build --release -p protocol

run-host:
	cargo run -p wasvy
