/* RISCulator - RISC-V Emulator */
/*   Utility functions here     */

// Libraries here
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use crate::Register;
use crate::RAM;
use crate::Vproc;
use std::thread;
use std::time::Duration;
use std::thread::spawn;
use std::fmt::Binary;
use colored::*;
use std::process::Command;
use std::process;

// Constants
//const CYCLES: u32 = 100;
const SPEED: usize = 1;
const XLEN: usize = 32;
const INI: usize = 4; // Address offset
const PROGRAM_LENGTH: usize = 1000;

// Static
static mut PROGRAM_LEN: usize = 0;      // UNSAFE not RUSTIC
static mut PC: isize = 0x0000;

// Logo displaying function
pub fn logo_display() {
    /* RISCulator logo */
    let filename = "src/assets/logo.txt";
    let logo_con = fs::read_to_string(filename)
        .expect("Failed to read the file");
    println!("{}", logo_con.yellow());
}

// Enumerators


// Boot sequence (non-OS)
pub fn boot_seq(xlen: usize, extension: &str, reg_size: usize, ram_size: usize) {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    log::info!("Boot Sequence Starting");
    log::info!("Loading configurations");
    log::info!("Instruction length: {}", xlen);
    log::info!("Extension: RV{}{}", xlen, extension);
    log::info!("RAM size: {}", ram_size);
}

// Register test. rut -> registers-under-test
pub fn register_tests(reg_size: usize, rut: &mut Register) {
    for i in 0..reg_size-1 {
        rut.write(i.try_into().unwrap(), 1);
        rut.read(i.try_into().unwrap());
        assert!(rut.read(i.try_into().unwrap()) == 1);
    }
    rut.reset();
}

// RAM test. rut -> RAM-under-test
pub fn ram_tests(ram_size: usize, rut: &mut RAM) {
    for i in 0..ram_size-1 {
        rut.write(i.try_into().unwrap(), 1);
        rut.read(i.try_into().unwrap());
        assert!(rut.read(i.try_into().unwrap()) == 1);
    }
    rut.reset();
}

pub fn line_splitter(line: &str) -> String {
    let mut line_vec = Vec::new();
    let mut line_mut = line.split("").collect::<Vec<_>>();
    for i in 0..8 {
        line_mut.remove(0);
        line_vec.push(line_mut[6]);
    }
    let line_vec_new = line_vec.join("");
    line_vec_new
}

// Program parsing function
pub fn program_parser(path: &str,  ram: &mut RAM) {
    let bin_file = File::open(path).unwrap();
     let reader = BufReader::new(bin_file);

     let mut bin_vec = Vec::new();
     let mut bin_only = Vec::new();

     for line in reader.lines() {
         bin_vec.push(line.unwrap());
     }

     for i in 0..7 {
         bin_vec.remove(0);
     }

     for i in 0..bin_vec.clone().len() {
         let mut temp = &mut bin_vec.clone()[i];
         let mut temp_line = line_splitter(temp);
         bin_only.push(temp_line);
     }

     for i in 0..bin_only.len() {
         //println!("{:?}", bin_only[i]);
         let temp_line = isize::from_str_radix(&bin_only[i], 16).unwrap();
         ram.write(i.try_into().unwrap(), temp_line);    // Error handling when overflow needs to be added here
         ram.address[i] = i*INI;
         unsafe{(PROGRAM_LEN = PROGRAM_LEN + 1)};
     }
 }

// stage2 -> Decode + Execute
pub fn stage2 (proc: &mut Vproc) {
    let mut instr: isize = 0;
    let mut cycles: isize = 0;
    proc.regs.write(2, 2147483632);

    for iter in 0..PROGRAM_LENGTH {
        let mut pcby4: usize = (unsafe{PC/4}).try_into().unwrap();
        if proc.ram_module.dirty_bit[pcby4] == 1 {
            instr = proc.ram_module.read(pcby4.try_into().unwrap());
            let mut instr_str = format!("{:032b}", instr).to_string();
            let mut instr_str_split = instr_str.split("").collect::<Vec<_>>();
            instr_str_split.remove(0);
            instr_str_split.remove(instr_str_split.len()-1);
            let mut temp = instruction_decoder(instr_str_split, proc, cycles);
            println!("{}", unsafe{PC});
            proc.update_regs(temp);
            proc.regs.print_dirty();
            cycles += 1;
        }
    }
}

// Instruction Decoder
pub fn instruction_decoder(instr: Vec<&str>, proc: &mut Vproc, cycles: isize) -> Register {
    /*
     * This decoder is based on the RISC-V Unprivileged Spec v2.2
     */

    //let mut return_vec: Vec<i32> = Vec::new();  // Return vector
    if instr.len() != XLEN {
        panic!("Wrong instruction length!");
    }

    /*
     * Instruction breakdown
     * 31 --------------------------------6------0
     * 0----------------------------------25------31
     * /                                  /opcode/
     */

    let mut temp_regs = Register::new();
    let mut temp_ram = RAM::new();

    let opcode_slice = &instr[25..];    // opcode field
    let opcode_slice_joined = opcode_slice.join("");

    log::info!("{}", "--------------------------------".green());

    match opcode_slice_joined.as_str() {
        "0000011" => {      // Load Instructions
            let funct3_slice = &instr[17..20];
            let funct3_slice_joined = funct3_slice.join("");
            let rd_slice = &instr[20..25];
            let rd_slice_joined = rd_slice.join("");
            let rs1_slice = &instr[12..17];
            let rs1_slice_joined = rs1_slice.join("");
            let mut imm_slice = &instr[0..12];
            let mut imm_slice_joined = imm_slice.join("");

            let rd_bits = isize::from_str_radix(&rd_slice_joined, 2).unwrap();
            let rs1_bits = isize::from_str_radix(&rs1_slice_joined, 2).unwrap();
            let mut imm_bits = isize::from_str_radix(&imm_slice_joined, 2).unwrap();

            // Immediate generator/handler
            if imm_slice[0] == "1" {
                let mut x = 1;
                while(true) {
                    let mut twos = isize::pow(2, x);
                    if (imm_bits as f32)/(twos as f32) < 1.0 {
                        imm_bits = imm_bits - twos;
                        break;
                    }
                    else {
                        x = x + 1;
                    }
                }
            }

            match funct3_slice_joined.as_str() {
                "000" => {      // Load Byte (8-bits)
                    log::info!("Load Byte (LB) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LB x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 + imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after LB operation   : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "001" => {      // Load Half-word (16-bits)
                    log::info!("Load Half-word (LH) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LH x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 + imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after LH operation   : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "010" => {      // Load Word (32-bits)
                    log::info!("Load Word (LW) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LW x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op2_sw = (rs1_bits + imm_bits) & 0xffff;
                    let mut op1 = proc.ram_module.read_from_addr(op2_sw.try_into().unwrap());
                    println!("{}", op2_sw);
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    temp_regs.write(rd_bits.try_into().unwrap(), op1);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "100" => {      // Load Byte Unsigned (u8-bits)
                    log::info!("Load Byte Unsigned (LBU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LBU x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 + imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after LBU operation  : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "101" => {      // Load Half-word Unsigned (u16-bits)
                    log::info!("Load Half-word Unsigned (LHU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LHU x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 + imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after LHU operation  : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                default => {
                    log::error!("Instruction format error!");
                    temp_regs
                }
            &_ => todo!()
            }
        }

        "0100011" => {      // Store Instructions
            let funct3_slice = &instr[17..20];
            let funct3_slice_joined = funct3_slice.join("");
            let rs2_slice = &instr[7..12];
            let rs2_slice_joined = rs2_slice.join("");
            let rs1_slice = &instr[12..17];
            let rs1_slice_joined = rs1_slice.join("");
            let mut imm_slice = &instr[0..7];
            let mut imm_slice_joined = imm_slice.join("");
            let imm2_slice = &instr[20..25];
            let imm2_slice_joined = imm2_slice.join("");

            imm_slice_joined = imm_slice_joined + &imm2_slice_joined;
            let rs1_bits = isize::from_str_radix(&rs1_slice_joined, 2).unwrap();
            let rs2_bits = isize::from_str_radix(&rs2_slice_joined, 2).unwrap();
            let mut imm_bits = isize::from_str_radix(&imm_slice_joined, 2).unwrap();

            // Immediate generator/handler
            if imm_slice[0] == "1" {
                let mut x = 1;
                while(true) {
                    let mut twos = isize::pow(2, x);
                    if (imm_bits as f32)/(twos as f32) < 1.0 {
                        imm_bits = imm_bits - twos;
                        break;
                    }
                    else {
                        x = x + 1;
                    }
                }
            }

            match funct3_slice_joined.as_str() {
                "000" => {      // Store Byte (8-bits)
                    log::info!("Store Byte (SB) instruction decoded");
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two address: x{}", rs2_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SB x{}, {}(x{})", rs2_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "001" => {      // Store Half-word (16-bit);
                    log::info!("Store Half-word (SH) instruction decoded");
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two address: x{}", rs2_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SH x{}, {}(x{})", rs2_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "010" => {      // Store Word (32-bit)
                    log::info!("Store Word (SW) instruction decoded");
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two address: x{}", rs2_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SW x{}, {}(x{})", rs2_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                    let mut op2_sw = rs1_bits + imm_bits;
                    op2_sw = (op2_sw & 0xffff);
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Register Two contents   : {:032b}", op2);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    temp_ram.write_to_addr(op2_sw.try_into().unwrap(), op2);
                    temp_ram.print_dirty();
                    proc.update_ram(temp_ram);
                    proc.ram_module.print_dirty();
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                default => {
                    log::error!("Instruction format error!");
                    temp_regs
                }
            &_ => todo!()
            }
        }

        "0010011" => {      // Immediate type instructions
            let funct3_slice = &instr[17..20];
            let funct3_slice_joined = funct3_slice.join("");
            let rd_slice = &instr[20..25];
            let rd_slice_joined = rd_slice.join("");
            let rs1_slice = &instr[12..17];
            let rs1_slice_joined = rs1_slice.join("");
            let imm_slice = &instr[0..12];
            let mut imm_slice_joined = imm_slice.join("");

            let rd_bits = isize::from_str_radix(&rd_slice_joined, 2).unwrap();
            let rs1_bits = isize::from_str_radix(&rs1_slice_joined, 2).unwrap();
            let mut imm_bits = isize::from_str_radix(&imm_slice_joined, 2).unwrap();

            // Immediate generator/handler
            if imm_slice[0] == "1" {
                let mut x = 1;
                while(true) {
                    let mut twos = isize::pow(2, x);
                    if  (imm_bits as f32)/(twos as f32) < 1.0 {
                        imm_bits = imm_bits - twos;
                        break;
                    }
                    else {
                        x = x + 1;
                    }
                }
            }

            match funct3_slice_joined.as_str() {
                "000" => {      // Add immediate
                    log::info!("Add Immediate (ADDI) instruction decoded");
                    log::info!("Destination Register address x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("ADDI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 + imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:012b}", imm_bits);
                    log::info!("RD after ADDI operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "010" => {      // Set less than immediate
                    log::info!("Set less than Immediate (SLTI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SLTI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = 0;
                    if op1 < imm_bits {
                        out = 1;
                    }
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after SLTI operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "011" => {      // Set less than immediate unsigned
                    log::info!("Set less than Immediate unsigned (SLTIU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SLTIU x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = 0;
                    if (op1 as u32) < (imm_bits as u32) {
                        out = 1;
                    }
                    log::info!("Register One contents    : {:032b}", op1);
                    log::info!("Immediate value          : {:032b}", imm_bits);
                    log::info!("RD after SLTIU operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "100" => {      // XOR Immediate
                    log::info!("XOR Immediate (XORI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("XORI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 ^ imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after XORI operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "110" => {      // OR Immediate
                    log::info!("OR Immediate (ORI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("ORI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 | imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after ORI operation  : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                "111" => {      // AND Immediate
                    log::info!("AND Immediate (ANDI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("ANDI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut out = op1 & imm_bits;
                    log::info!("Register One contents   : {:032b}", op1);
                    log::info!("Immediate value         : {:032b}", imm_bits);
                    log::info!("RD after ANDI operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};
                    temp_regs
                }
                default => {
                    log::error!("Instruction format error!");
                    panic!("Stage 2 failed due to error!");
                    temp_regs
                }
            &_ => todo!()
            }
        }

        "0110111" => {      // Load upper immediate
            let rd_slice = &instr[20..25];
            let rd_slice_joined = rd_slice.join("");
            let imm_slice = &instr[0..20];
            let imm_slice_joined = imm_slice.join("");

            let rd_bits = isize::from_str_radix(&rd_slice_joined, 2).unwrap();
            let mut imm_bits = isize::from_str_radix(&imm_slice_joined, 2).unwrap();

            // Immediate generator/handler
            if imm_slice[0] == "1" {
                let mut x = 1;
                while(true) {
                    let mut twos = isize::pow(2, x);
                    if  (imm_bits as f32)/(twos as f32) < 1.0 {
                        imm_bits = imm_bits - twos;
                        break;
                    }
                    else {
                        x = x + 1;
                    }
                }
            }

            log::info!("Load Upper Immediate (LUI) instruction decoded");
            log::info!("Destination Register address: x{}", rd_bits);
            log::info!("Immediate address: {}", imm_bits);
            log::info!("LUI x{}, {}", rd_bits, imm_bits);
            log::info!("{}", "--------------------------------".green());

            /* Execution step */
            let mut out = imm_bits >> 12;
            log::info!("RD after LUI operation : {:032b}", out);
            temp_regs.write(rd_bits.try_into().unwrap(), out);
            unsafe{PC += 0x0004}      // Program counter
            temp_regs
        }

        "0010111" => {      // Add upper immediate with PC
            let rd_slice = &instr[20..25];
            let rd_slice_joined = rd_slice.join("");
            let imm_slice = &instr[0..20];
            let imm_slice_joined = imm_slice.join("");

            let rd_bits = isize::from_str_radix(&rd_slice_joined, 2).unwrap();
            let mut imm_bits = isize::from_str_radix(&imm_slice_joined, 2).unwrap();

            // Immediate generator/handler
            if imm_slice[0] == "1" {
                let mut x = 1;
                while(true) {
                    let mut twos = isize::pow(2, x);
                    if  (imm_bits as f32)/(twos as f32) < 1.0 {
                        imm_bits = imm_bits - twos;
                        break;
                    }
                    else {
                        x = x + 1;
                    }
                }
            }

            log::info!("Add upper immediate with PC (AUIPC) instruction decoded");
            log::info!("Destination Register address: x{}", rd_bits);
            log::info!("Immediate address: {}", imm_bits);
            log::info!("AUIPC x{}, {}", rd_bits, imm_bits);
            log::info!("{}", "--------------------------------".green());

            /* Execution step */
            unsafe{PC += (imm_bits >> 12)};
            let mut out = unsafe{PC} + 0x0004;
            log::info!("RD after AUIPC operation : {:032b}", out);
            temp_regs.write(rd_bits.try_into().unwrap(), out);
            unsafe{PC += 0x0004};
            temp_regs
        }

        "1101111" => {      // Jump and link
            let rd_slice = &instr[20..25];
            let rd_slice_joined = rd_slice.join("");
            let imm_slice = &instr[0..20];
            let imm_slice_joined = imm_slice.join("");
            let imm_slice_1 = imm_slice[0].to_string();        // imm[19]
            //let imm_slice_1_joined = imm_slice_1.join("");
            let imm_slice_2 = &imm_slice[1..10];    // imm[7:0]
            let imm_slice_2_joined = imm_slice_2.join("");
            let imm_slice_3 = imm_slice[10].to_string();       // imm[8]
           // let imm_slice_3_joined = imm_slice_3.join("");
            let imm_slice_4 = &imm_slice[11..20];   // imm[18:9]
            let imm_slice_4_joined = imm_slice_4.join("");
            let mut imm_final = imm_slice_2_joined + &imm_slice_3 + &imm_slice_4_joined + &imm_slice_1;

            let rd_bits = isize::from_str_radix(&rd_slice_joined, 2).unwrap();
            let mut imm_bits = isize::from_str_radix(&imm_final, 2).unwrap();

            log::info!("Jump and Link (JAL) instruction decoded");
            log::info!("Destination Register address: x{}", rd_bits);
            log::info!("Immediate address: x{}", imm_bits);
            log::info!("JAL x{}, {}", rd_bits, imm_bits);
            log::info!("{}", "--------------------------------".green());

            /* Execution step */
            unsafe{PC += 0x0004};
            let mut out = unsafe{PC} + 0x0004;
            unsafe{PC += imm_bits};
            log::info!("PC after JAL operation : {:032b}", unsafe{PC});
            temp_regs.write(rd_bits.try_into().unwrap(), out);
            temp_regs
        }

        "1100111" => {      // Jump and link to register
            let rd_slice = &instr[20..25];
            let rd_slice_joined = rd_slice.join("");
            let imm_slice = &instr[0..12];
            let imm_slice_joined = imm_slice.join("");
            let rs1_slice = &instr[12..17];
            let rs1_slice_joined = rs1_slice.join("");

            let rd_bits = isize::from_str_radix(&rd_slice_joined, 2).unwrap();
            let rs1_bits = isize::from_str_radix(&rs1_slice_joined, 2).unwrap();
            let mut imm_bits = isize::from_str_radix(&imm_slice_joined, 2).unwrap();

            // Immediate generator/handler
            if imm_slice[0] == "1" {
                let mut x = 1;
                while(true) {
                    let mut twos = isize::pow(2, x);
                    if  (imm_bits as f32)/(twos as f32) < 1.0 {
                        imm_bits = imm_bits - twos;
                        break;
                    }
                    else {
                        x = x + 1;
                    }
                }
            }

            log::info!("Jump and Link to register (JALR) instruction decoded");
            log::info!("Destination Register address: x{}", rd_bits);
            log::info!("Register one address: x{}", rs1_bits);
            log::info!("Immediate value: {}", imm_bits);
            log::info!("JALR x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
            log::info!("{}", "--------------------------------".green());

            /* Execution step */
            unsafe{PC += 0x0004};
            let mut out = unsafe{PC};
            unsafe{PC = (rs1_bits + imm_bits & !1)};
            log::info!("RD after JALR operation : {:032b}", out);
            log::info!("PC after JALR operation : {:032b}", unsafe{PC});
            temp_regs.write(rd_bits.try_into().unwrap(), out);

            if rd_bits == 0 && rs1_bits == 1 && imm_bits == 0 {
                log::info!("Execution successful!");
                println!("Cycles: {:?}", &cycles);
                process::exit(0);
            }
            temp_regs
        }

        "0110011" => {      // Arithmetic instructions
            let funct3_slice = &instr[17..20];
            let funct3_slice_joined = funct3_slice.join("");
            let funct7_slice = &instr[0..7];
            let funct7_slice_joined = funct7_slice.join("");
            let rs2_slice = &instr[7..12];
            let rs2_slice_joined = rs2_slice.join("");
            let rs1_slice = &instr[12..17];
            let rs1_slice_joined = rs1_slice.join("");
            let rd_slice = &instr[20..25];
            let rd_slice_joined = rd_slice.join("");

            let rs1_bits = isize::from_str_radix(&rs1_slice_joined, 2).unwrap();
            let rs2_bits = isize::from_str_radix(&rs2_slice_joined, 2).unwrap();
            let mut rd_bits = isize::from_str_radix(&rd_slice_joined, 2).unwrap();

            match funct3_slice_joined.as_str() {
                "000" => {
                    match funct7_slice_joined.as_str() {
                        "0000000" => {      // Add
                            log::info!("Addition (ADD) instruction decoded");
                            log::info!("Destination Register address: x{}", rd_bits);
                            log::info!("Register One address: x{}", rs1_bits);
                            log::info!("Register Two value: {}", rs2_bits);
                            log::info!("ADD x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                            log::info!("{}", "--------------------------------".green());

                            /* Execution step */
                            let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                            let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                            let mut out = op1 + op2;
                            log::info!("Register One contents  : {:032b}", op1);
                            log::info!("Register Two contents  : {:032b}", op2);
                            log::info!("RD after ADD operation : {:032b}", out);
                            temp_regs.write(rd_bits.try_into().unwrap(), out);
                            unsafe{PC += 0x0004}      // Program counter
                            temp_regs
                        }
                        "0100000" => {      // Sub
                            log::info!("Subtraction (SUB) instruction decoded");
                            log::info!("Destination Register address: x{}", rd_bits);
                            log::info!("Register One address: x{}", rs1_bits);
                            log::info!("Register Two value: {}", rs2_bits);
                            log::info!("SUB x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                            log::info!("{}", "--------------------------------".green());

                            /* Execution step */
                            let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                            let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                            let mut out = op1 - op2;
                            log::info!("Register One contents  : {:032b}", op1);
                            log::info!("Register Two contents  : {:032b}", op2);
                            log::info!("RD after SUB operation : {:032b}", out);
                            temp_regs.write(rd_bits.try_into().unwrap(), out);
                            unsafe{PC += 0x0004};      // Program counter
                            temp_regs
                        }
                        &_ => todo!()
                    }
                }
                "001" => {      // Shift left logical
                    log::info!("Shift Left Logical (SLL) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two value: {}", rs2_bits);
                    log::info!("SLL x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                    let mut out = op1 << op2;
                    log::info!("Register One contents         : {:032b}", op1);
                    log::info!("Register Two contents         : {:032b}", op2);
                    log::info!("RD after Shift Left operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};      // Program counter
                    temp_regs
                }
                "010" => {      // Set less than
                    log::info!("Set less than (SLT) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two value: {}", rs2_bits);
                    log::info!("SLT x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                    let mut out = 0;
                    if op1 < op2  {
                        out = 1;
                    }
                    log::info!("Register One contents            : {:032b}", op1);
                    log::info!("Register Two contents            : {:032b}", op2);
                    log::info!("RD after Set less than operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};       // Program counter
                    temp_regs
                }
                "011" => {      // Set less than unsigned
                    log::info!("Set less than unsigned (SLTU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two value: {}", rs2_bits);
                    log::info!("SLTU x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                    let mut out = 0;
                    if (op1 as u32) < (op2 as u32) {
                        out = 1;
                    }
                    log::info!("Register One contents              : {:032b}", op1);
                    log::info!("Register Two contents              : {:032b}", op2);
                    log::info!("RD after Set less than U operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};       // Program counter
                    temp_regs
                }
                "100" => {      // XOR
                    log::info!("XOR instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two value: {}", rs2_bits);
                    log::info!("XOR x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                    let mut out = op1 ^ op2;
                    log::info!("Register One contents  : {:032b}", op1);
                    log::info!("Register Two contents  : {:032b}", op2);
                    log::info!("RD after XOR operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};       // Program counter
                    temp_regs
                }
                "101" => {      // Shift right
                    match funct7_slice_joined.as_str() {
                        "0000000" => {      // Shift right logical
                            log::info!("Shift Right Logical (SRL) instruction decoded");
                            log::info!("Destination Register address: x{}", rd_bits);
                            log::info!("Register One address: x{}", rs1_bits);
                            log::info!("Register Two value: {}", rs2_bits);
                            log::info!("SRL x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                            log::info!("{}", "--------------------------------".green());
                            unsafe{PC += 0x0004};
                            temp_regs
                        }
                        "0100000" => {      // Shift right arithmetic
                            log::info!("Shift Right Arithmetic (SRA) instruction decoded");
                            log::info!("Destination Register address: x{}", rd_bits);
                            log::info!("Register One address: x{}", rs1_bits);
                            log::info!("Register Two value: {}", rs2_bits);
                            log::info!("SRA x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                            log::info!("{}", "--------------------------------".green());
                            unsafe{PC += 0x0004};
                            temp_regs
                        }
                        &_ => todo!()
                    }
                }
                "110" => {      // OR
                    log::info!("OR instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two value: {}", rs2_bits);
                    log::info!("OR x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                    let mut out = op1 | op2;
                    log::info!("Register One contents  : {:032b}", op1);
                    log::info!("Register Two contents  : {:032b}", op2);
                    log::info!("RD after OR operation  : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};       // Program counter
                    temp_regs
                }
                "111" => {      // AND
                    log::info!("AND instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two value: {}", rs2_bits);
                    log::info!("AND x{}, x{}, x{}", rd_bits, rs1_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());

                    /* Execution step */
                    let mut op1 = proc.regs.read(rs1_bits.try_into().unwrap());
                    let mut op2 = proc.regs.read(rs2_bits.try_into().unwrap());
                    let mut out = op1 & op2;
                    log::info!("Register One contents  : {:032b}", op1);
                    log::info!("Register Two contents  : {:032b}", op2);
                    log::info!("RD after AND operation : {:032b}", out);
                    temp_regs.write(rd_bits.try_into().unwrap(), out);
                    unsafe{PC += 0x0004};       // Program counter
                    temp_regs
                }
            &_ => todo!()
            }
        }
        default => {
            log::error!("Opcode not found!");
            temp_regs
        }
    &_ => todo!()
    }
}
