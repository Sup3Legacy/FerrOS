all: disk run

disk:
	python3 ./disk/createDisk.py

ferros_release: fmt
	cargo build --release

ferros:
	cargo build

run: fmt
	cargo run --release

fmt:
	cargo fmt

doc: fmt
	cargo doc --document-private-items --open

clean: fmt
	cargo clean

sound: fmt
	cargo run -- -soundhw pcspk -drive format=raw,file=disk/disk.disk,index=2 -boot c

count: fmt
	wc -l `find src -type f` -l `find disk -type f`

memory: fmt
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk/disk.img,index=2 -boot c

memory2: fmt
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk/disk.disk,index=2 -boot c

.PHONY: all run fmt doc clean count sound memory memory2 disk
