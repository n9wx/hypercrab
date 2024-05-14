TARGET		:= riscv64gc-unknown-none-elf
MODE		?= release
KERNEL_ELF 	:= target/$(TARGET)/$(MODE)/hypercrab
KERNEL_BIN 	:= target/$(TARGET)/$(MODE)/hypercrab.bin
KERNEL_ENTRY_PA := 0x80200000

CPUS		:= 1

BOOTLOADER	:= bootloader/rustsbi-qemu.bin
QEMU		:= qemu-system-riscv64

QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
QEMUOPTS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
QEMUOPTS	+=-device virtio-keyboard-device
QEMUOPTS	+=-device virtio-mouse-device

OBJCOPY		:= rust-objcopy --binary-architecture=riscv64
OBJDUMP		:= rust-objdump --arch-name=riscv64

$(KERNEL_BIN):build
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

build:
	cargo build --release
	#cargo build

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
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'