# RISCulator
RISCulator is a lightweight RISC-V emulator.

# Wiki

## Installation
- Install the Rust compiler 'rustc' using rustup.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- Clone this repo 

```bash
git clone https://github.com/suhaskv1/RISCulator.git
```
- Change directory to RISCulator/src and run the following

```bash
cd RISCulator/src
cargo run
```

### Checklist
- Simple implementation of RV32I.
- Implement other extensions - M,A,F.
- Linux compatible.
