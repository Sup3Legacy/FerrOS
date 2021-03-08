
all:
	cargo run -- -drive format=raw,file=disk.disk,index=2 -boot c

clean:
	cargo clean

sound:
	cargo run -- -soundhw pcspk -drive format=raw,file=disk.disk,index=2 -boot c

count:
	wc -l `find src -type f`

memory:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk.img,index=2 -boot c

memory2:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferros/debug/bootimage-ferr_os.bin	-drive format=raw,file=disk.disk,index=2 -boot c