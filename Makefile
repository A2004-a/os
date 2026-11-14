.PHONY: all build image run debug test test-qemu clean

all: build

build:
	cargo build --package novaos-apex-boot --target x86_64-unknown-uefi
	cargo build --package novaos-apex-kernel --target x86_64-unknown-none
	cargo build --package novaos-apex-runtime --target x86_64-unknown-none
	cargo build --package novaos-apex-init --target x86_64-unknown-none
	cargo build --package novaos-apex-shell --target x86_64-unknown-none
	cargo build --package novaos-apex-coreutils --target x86_64-unknown-none
	cargo build --package novaos-apex-toolkit --target x86_64-unknown-none
	cargo build --package novaos-apex-apps --target x86_64-unknown-none

image:
	@mkdir -p build/EFI/BOOT
	cargo build --package novaos-apex-boot --target x86_64-unknown-uefi
	cargo build --package novaos-apex-kernel --target x86_64-unknown-none
	cp -f target/x86_64-unknown-uefi/debug/novaos-apex-boot.efi build/EFI/BOOT/BOOTX64.EFI
	cp -f target/x86_64-unknown-none/debug/novaos-apex-kernel build/novaos-apex.elf

run: image
	./tools/run-qemu

debug: image
	./tools/debug-gdb

test:
	cargo check --workspace --target x86_64-unknown-none
	cargo check --package novaos-apex-boot --target x86_64-unknown-uefi

test-qemu: image
	./tools/test-qemu

clean:
	cargo clean
	rm -rf build
	rm -f serial.log
