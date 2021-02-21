
all:
	cargo run

clean:
	cargo clean

sound:
	cargo run -- -soundhw pcspk

count:
	wc -l `find src -type f`