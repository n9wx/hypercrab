TARGET		:= riscv64gc-unknown-none-elf
MODE		?= release
TARGET_FILE ?= hypervisor
KERNEL_ELF 	:= target/$(TARGET)/$(MODE)/hypercrab
KERNEL_BIN 	:= target/$(TARGET)/$(MODE)/hypercrab.bin
KERNEL_ENTRY_PA := 0x80200000

CPUS		:= 1

GUEST_BIN := "guest.bin"
GUEST_ELF := "guest.elf"

TARGET_BIN ?= $(KERNEL_BIN)
TARGET_ELF ?= $(KERNEL_ELF)
TARGET_DIR ?= "src"

ifeq ($(TARGET_FILE),guest)
TARGET_BIN = $(GUEST_BIN)
TARGET_ELF = $(GUEST_ELF)
TARGET_DIR = "guest/rCore-Tutorial-v3/os"
endif

BOOTLOADER	:= bootloader/rustsbi-qemu.bin
QEMU		:= qemu-system-riscv64

QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
QEMUOPTS	+=-device loader,file=$(TARGET_BIN),addr=$(KERNEL_ENTRY_PA)
QEMUOPTS	+=-device virtio-keyboard-device
QEMUOPTS	+=-device virtio-mouse-device

OBJCOPY		:= rust-objcopy --binary-architecture=riscv64
OBJDUMP		:= rust-objdump --arch-name=riscv64

$(KERNEL_BIN):build
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

CARGO_OPTS ?= build --release

ifeq ($(MODE),debug)
CARGO_OPTS = build
endif

clean:
	cargo clean

build:
	cargo $(CARGO_OPTS)

run: $(KERNEL_BIN)
	$(QEMU) $(QEMUOPTS)

debug: $(KERNEL_BIN)
	@tmux new-session -d \
		"$(QEMU) $(QEMUOPTS) -s -S" && \
		tmux split-window -h "$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: $(KERNEL_BIN)
	$(QEMU) $(QEMUOPTS) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb --directory=$(TARGET_DIR) -ex 'file $(TARGET_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'