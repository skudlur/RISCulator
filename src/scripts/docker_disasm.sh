#!/bin/sh

set -eu

TRIPLET=riscv32-unknown-linux-gnu

docker run \
  -v ./src:/src \
  -v ./target:/artifact \
  --rm -i dockcross/linux-riscv32 \
  sh - <<EOF
cd /artifact
${TRIPLET}-cc -march=rv32id -c /src/test/main.c -o main.o
${TRIPLET}-objcopy -O binary -j .text main.o binfile
${TRIPLET}-objdump -d main.o > out.txt

# Hack to make the created files and dirs rw by all users
# since docker creates files in mounted volumes as root...
chmod 777 -R /artifact
EOF
