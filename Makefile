build:
	cargo build

run:
	cargo run

count:
	wc -l src/*.rs gui/*.ts gui/*.tsx

watch:
	node ./build.js watch
check:
	npx tsc -w

export:
	sox -r 44100 -c 2 /tmp/a.sw /tmp/a.wav
	oggenc /tmp/a.wav
	mv /tmp/a.ogg ~/tmp/a.ogg
