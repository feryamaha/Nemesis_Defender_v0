CLANG ?= clang
ARCH_INCLUDE ?= /usr/include/$(shell uname -m)-linux-gnu
VMLINUX_H   ?= ebpf/vmlinux.h
BPF_CFLAGS  := -target bpf -O2 -g -Iinclude -Iebpf -I$(ARCH_INCLUDE)

all: $(VMLINUX_H) ebpf/nemesis-block.bpf.o

$(VMLINUX_H):
	bpftool btf dump file /sys/kernel/btf/vmlinux format c > $@

ebpf/nemesis-block.bpf.o: ebpf/nemesis-block.bpf.c include/nemesis_maps.h $(VMLINUX_H)
	$(CLANG) $(BPF_CFLAGS) -c $< -o $@

clean:
	rm -f ebpf/nemesis-block.bpf.o ebpf/vmlinux.h
