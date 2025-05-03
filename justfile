build-protocol:
	cargo component build --release -p protocol

build-example example:
	cargo component build --release -p {{example}}

build-example-python:
	cd examples/python_example/src/python_example && poetry run componentize-py --wit-path ../../wit/ --world guest componentize app -o ../../../../python.wasm

# Create the bindings for the python example
example-bindings-python:
	cd examples/python_example && poetry run componentize-py --wit-path wit/ --world guest bindings src/python_example

# For the fetching to take effect you must delete the deps folder manually
example-fetch-deps example:
	cd examples/{{example}} && wkg wit fetch

run-host:
	cargo run -p wasvy

build-wasvy-ecs:
	wkg wit build --wit-dir ./wit/ecs/

#wkg publish --package wasvy:ecs@0.0.1 wasvy:ecs.wasm --registry wa.dev
publish-wasvy-ecs file_path version:
	wkg publish --package wasvy:ecs@{{version}} {{file_path}} --registry wa.dev
