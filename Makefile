all: disk ferros_release

disk:
	python3 ./disk/createDisk.py

ferros_release:
	cargo build --release

ferros:
	cargo build

run:
	cargo run --release

fmt:
	cargo fmt

doc: fmt
	cargo doc --document-private-items --open

clean: fmt
	cargo clean

sound:
	cargo run -- -soundhw pcspk -drive format=raw,file=disk/disk.disk,index=2 -boot c

count: fmt
	wc -l `find src -type f` -l `find disk -type f`

memory:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk/disk.img,index=2 -boot c

memory2:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk/disk.disk,index=2 -boot c

config:
	rustup override nightly
	cargo install bootimage
	rustup component add llvm-tools-preview
	rustup component ad rust-src

.PHONY: all run fmt doc clean count sound memory memory2 disk
