CC=clang
BIT_INT_FLAGS=-Xclang -fexperimental-max-bitint-width=512
CFLAGS=$(TARGET_FLAGS) -emit-llvm -O3 -ffreestanding -fno-builtin -Wall -Wno-unused-function $(BIT_INT_FLAGS)

../target/bpf/%.bc: %.c
	$(CC) -c $(CFLAGS) $< -o $@

../target/wasm/%.bc: %.c
	$(CC) -c $(CFLAGS) $< -o $@

../target/riscv/%.bc: %.c
	$(CC) -c $(CFLAGS) $< -o $@

SOLANA=$(addprefix ../target/bpf/,solana.bc bigint.bc format.bc stdlib.bc ripemd160.bc heap.bc)
WASM=$(addprefix ../target/wasm/,ripemd160.bc stdlib.bc bigint.bc format.bc heap.bc)
RISCV=$(addprefix ../target/riscv/,polkavm_guest.bc ripemd160.bc stdlib.bc bigint.bc format.bc heap.bc)

all: $(SOLANA) $(WASM)
riscv: $(RISCV)
Wasm: $(WASM)

$(SOLANA) $(WASM) $(RISCV): | outputs_dirs

$(SOLANA): TARGET_FLAGS=--target=sbf
$(WASM): TARGET_FLAGS=--target=wasm32
$(RISCV): TARGET_FLAGS=--target=riscv32 -march=rv32em -mabi=ilp32e -fforce-enable-int128 -fno-exceptions

bpf/solana.bc: solana.c solana_sdk.h | outputs_dirs
riscv/polkavm_guest.bc: polkavm_guest.c polkavm_guest.h | outputs_dirs

outputs_dirs:
	@mkdir -p ../target/bpf ../target/wasm ../target/riscv

clean:
	rm -rf ../target/bpf ../target/wasm ../target/riscv

test:
	clang -DTEST -DSOL_TEST -O3 -Wall solana.c stdlib.c -o test

lint:
	clang-format *.c *.h --style=file --dry-run -Werror
