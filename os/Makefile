# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin

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
export WATCH_PATH=../tests/user/build/
export SOURCE_PATH=../tests/user/build/elf/
export TARGET_PATH=../tests/user/build/elf/
export FILE_EXT=.elf
endif

build:
	@cargo build --release
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

clean:
	@cargo clean

run: gen-user build qemu

test: gen-test build qemu

gen-user:
	@cd ../user && make build

gen-test: 
	@git submodule init
	@git submodule update
	@cd ../tests/user && make clean && make all CHAPTER=$(CHAPTER)

qemu:
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

.PHONY: build clean qemu test