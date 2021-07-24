# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
APPS := ../user/src/bin/*

# BOARD
BOARD ?= qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

KERNEL_ENTRY_PA := 0x80200000

# KERNEL OPTIONS
LOG ?= 

# TEST OPTIONS
ifdef CHAPTER
FS_IMG := ../tests/user/build/elf/fs.img
SOURCE_PATH := ../tests/user/build/bin/
TARGET_PATH := ../tests/user/build/elf/
FILE_EXT := --ext .elf
else
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
SOURCE_PATH := ../user/src/bin/
TARGET_PATH := ../user/target/$(TARGET)/$(MODE)/
FILE_EXT :=
endif

build:
	@cargo build --release
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

clean:
	@cargo clean
	@cd ../user && make clean
	@cd ../tests/user && make clean

$(FS_IMG):
	@cd ../easy-fs-fuse && cargo run --release -- -s $(SOURCE_PATH) -t $(TARGET_PATH) $(FILE_EXT)

run: build gen-user $(FS_IMG) qemu

test: build gen-test $(FS_IMG) qemu

gen-user:
	@cd ../user && make build

gen-test: 
	@git submodule init
	@git submodule update
	@cd ../tests/user && make all CHAPTER=$(CHAPTER)

qemu:
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

.PHONY: build clean qemu test