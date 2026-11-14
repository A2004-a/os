CARGO ?= cargo
QEMU ?= qemu-system-x86_64
OVMF ?= /usr/share/ovmf/OVMF.fd

.PHONY: all build image run debug test test-qemu clean

all: build

build:
	$(CARGO) build --package novaos-uefi-boot --target x86_64-unknown-uefi
	$(CARGO) build --package novaos-kernel --target x86_64-unknown-none

image: build
	mkdir -p build/EFI/BOOT
	cp target/x86_64-unknown-uefi/debug/novaos-uefi-boot.efi build/EFI/BOOT/BOOTX64.EFI || true
	cp target/x86_64-unknown-none/debug/novaos-kernel build/novaos.elf || true
	cp novaos.cfg build/novaos.cfg || true

run: image
	$(QEMU) -bios $(OVMF) -drive format=raw,file=fat:rw:build -serial stdio -m 512M -device isa-debug-exit,iobase=0xf4,iosize=0x04

debug: image
	$(QEMU) -bios $(OVMF) -drive format=raw,file=fat:rw:build -serial stdio -m 512M -s -S

test-qemu: image
	$(QEMU) -bios $(OVMF) -drive format=raw,file=fat:rw:build -serial stdio -display none -m 512M -device isa-debug-exit,iobase=0xf4,iosize=0x04

test:
	$(CARGO) test --package novaos-kernel --lib

clean:
	$(CARGO) clean
	rm -rf build
