build:
	cargo build

run:
	cargo run

count:
	wc -l src/*.rs gui/*.ts{x,}

watch:
	node ./build.js watch
