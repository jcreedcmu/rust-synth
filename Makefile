build:
	cargo build

run:
	cargo run

count:
	wc -l src/*.rs

dyn/dyn.so:
	gcc -shared -rdynamic ./dyn/dyn.c  -o ./dyn/dyn.so
