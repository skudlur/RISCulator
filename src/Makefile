# RISCulator Makefile

# compiler
CC=riscv32-unknown-elf-gcc
COBJCPY=riscv32-unknown-elf-objcopy
COBJDMP=riscv32-unknown-elf-objdump

all: clean compile run

compile: test/main.c
	$(CC) -march=rv32i -c test/main.c -o test/main.o
	$(COBJCPY) -O binary -j .text test/main.o test/binfile
	$(COBJDMP) -d test/main.o > test/out.txt

run:
	cargo run

clean:
	rm -rf test/*.o
	rm -rf test/*.s
	rm -rf test/binfile
	rm -rf test/out.txt
