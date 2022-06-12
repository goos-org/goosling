default: all

all: iso

setup:
	mkdir -p build

iso: kernel limine
	rm -rf build/iso
	mkdir build/iso
	mkdir build/iso/boot
	cp build/kernel build/iso/boot/kernel
	cp build/limine/limine.sys build/iso/boot/limine.sys
	cp src/limine.cfg build/iso/boot/limine.cfg
	cp build/limine/limine-cd-efi.bin build/iso/boot/limine-cd-efi.bin
	cp build/limine/limine-cd.bin build/iso/boot/limine-cd.bin
	cd build && xorriso -as mkisofs -b boot/limine-cd.bin \
			-no-emul-boot -boot-load-size 4 -boot-info-table \
			--efi-boot boot/limine-cd-efi.bin \
			-efi-boot-part --efi-boot-image --protective-msdos-label \
			iso -o goosling.iso
	build/limine/limine-deploy build/goosling.iso

kernel: setup
	cargo build
	cp target/x86_64-goosling/debug/goosling build/kernel

limine: setup
	rm -rf build/limine
	cd build && git clone --branch v3.5.3-binary https://github.com/limine-bootloader/limine.git
	cd build/limine && make limine-deploy
