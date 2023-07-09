# RISCulator Makefile

DOCKER_IMAGE = dockcross/linux-riscv32

DISASSEMBLY = target/out.txt
DOCKER_DISASM_SCRIPT = src/scripts/docker_disasm.sh

all: risculator $(DISASSEMBLY) run

$(DISASSEMBLY): src/test/main.c $(DOCKER_DISASM_SCRIPT)
	$(DOCKER_DISASM_SCRIPT)

risculator: $(DISASSEMBLY)
	cargo build

run: risculator $(DISASSEMBLY)
	cargo run -- $(DISASSEMBLY)

clean:
	rm -rf target
