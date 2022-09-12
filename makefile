.PHONY: all
all: goosling.iso

.PHONY: all-hdd
all-hdd: goosling.hdd

.PHONY: run
run: goosling.iso
	qemu-system-x86_64 -M q35 -m 2G -cdrom goosling.iso -boot d

.PHONY: run-uefi
run-uefi: ovmf-x64 goosling.iso
	qemu-system-x86_64 -M q35 -m 2G -bios ovmf-x64/OVMF.fd -cdrom goosling.iso -boot d

.PHONY: run-hdd
run-hdd: goosling.hdd
	qemu-system-x86_64 -M q35 -m 2G -hda goosling.hdd

.PHONY: run-hdd-uefi
run-hdd-uefi: ovmf-x64 goosling.hdd
	qemu-system-x86_64 -M q35 -m 2G -bios ovmf-x64/OVMF.fd -hda goosling.hdd

ovmf-x64:
	mkdir -p ovmf-x64
	cd ovmf-x64 && curl -o OVMF-X64.zip https://efi.akeo.ie/OVMF/OVMF-X64.zip && 7z x OVMF-X64.zip

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v4.x-branch-binary --depth=1
	make -C limine

.PHONY: kernel
kernel:
	cargo build
	mkdir -p kernel/
	cp target/x86_64-freestanding/debug/goosling kernel/kernel.elf

goosling.iso: limine kernel
	rm -rf iso_root
	mkdir -p iso_root
	cp kernel/kernel.elf \
		limine.cfg limine/limine.sys limine/limine-cd.bin limine/limine-cd-efi.bin iso_root/
	xorriso -as mkisofs -b limine-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-cd-efi.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o goosling.iso
	limine/limine-deploy goosling.iso
	rm -rf iso_root

goosling.hdd: limine kernel
	rm -f goosling.hdd
	dd if=/dev/zero bs=1M count=0 seek=64 of=goosling.hdd
	parted -s goosling.hdd mklabel gpt
	parted -s goosling.hdd mkpart ESP fat32 2048s 100%
	parted -s goosling.hdd set 1 esp on
	limine/limine-deploy goosling.hdd
	sudo losetup -Pf --show goosling.hdd >loopback_dev
	sudo mkfs.fat -F 32 `cat loopback_dev`p1
	mkdir -p img_mount
	sudo mount `cat loopback_dev`p1 img_mount
	sudo mkdir -p img_mount/EFI/BOOT
	sudo cp -v kernel/kernel.elf limine.cfg limine/limine.sys img_mount/
	sudo cp -v limine/BOOTX64.EFI img_mount/EFI/BOOT/
	sync
	sudo umount img_mount
	sudo losetup -d `cat loopback_dev`
	rm -rf loopback_dev img_mount

.PHONY: clean
clean:
	rm -rf iso_root kernel goosling.iso goosling.hdd

.PHONY: distclean
distclean: clean
	rm -rf limine ovmf-x64
