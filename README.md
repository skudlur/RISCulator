<p align="center">
  <img src="https://github.com/skudlur/RISCulator/blob/main/src/assets/RISCulator_logo_gh.png" />
</p>

## Wiki
To be updated. WIP.

## Installation

### RISC-V GCC Compiler installation

#### Option 1:
- Head to https://github.com/riscv-collab/riscv-gnu-toolchain and build the compiler.

#### Option 2:
- Head to https://www.embecosm.com/resources/tool-chain-downloads/ and install a pre-built distro-specific compiler image and add the bin directory to PATH.

**Note:** Make sure you install both 32-bit and 64-bit compilers. RISCulator will support both instruction length emulation. 

### RISCulator installation
- Install the Rust compiler 'rustc' using rustup.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- Clone this repo 

```bash
git clone https://github.com/skudlur/RISCulator.git
```
- Change directory to RISCulator/src and run the following

```bash
cd RISCulator/src
make all
```

### Checklist
- Simple implementation of RV32I.
- Implement other extensions - M,A,F.
- Multi-core
