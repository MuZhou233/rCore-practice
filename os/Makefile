# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
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

SOURCE_PATH := ../user/src/bin/
TARGET_PATH := ../user/target/riscv64gc-unknown-none-elf/release/
# TEST OPTIONS
ifdef CHAPTER
SOURCE_PATH := ../tests/user/build/elf/
TARGET_PATH := ../tests/user/build/elf/
export WATCH_PATH=../tests/user/build/
export SOURCE_PATH=$(SOURCE_PATH)
export TARGET_PATH=$(TARGET_PATH)
export FILE_EXT=.elf
endif

build:
	@cargo build --release
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

clean:
	@cargo clean
	@cd ../user && make clean
	@cd ../tests/user && make clean

$(FS_IMG): $(APPS)
	@cd ../easy-fs-fuse && cargo run --release -- -s $(SOURCE_PATH) -t $(TARGET_PATH)

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