
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

// Constants
const CYCLES: u32 = 100;
const SPEED: usize = 1;
const PROGRAM_LEN: usize = 100; // replaced soon by calculating program length with a fn
const XLEN: usize = 32;

// Logo displaying function
pub fn logo_display() {
    /* RISCulator logo */
    let filename = "assets/logo.txt";
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
    thread::sleep(Duration::from_millis(10));
    log::info!("Loading configurations");
    thread::sleep(Duration::from_millis(10));
    log::info!("Instruction length: {}", xlen);
    thread::sleep(Duration::from_millis(10));
    log::info!("Extension: RV{}{}", xlen, extension);
    thread::sleep(Duration::from_millis(10));
    log::info!("RAM size: {}", ram_size);
    thread::sleep(Duration::from_millis(10));
}

// Register test. rut -> registers-under-test
pub fn register_tests(reg_size: usize, rut: &mut Register) {
    for i in 0..reg_size-1 {
        rut.write(i.try_into().unwrap(),1);
        rut.read(i.try_into().unwrap());
        assert!(rut.read(i.try_into().unwrap()) == 1);
    }
    rut.reset();
}

// RAM test. rut -> RAM-under-test
pub fn ram_tests(ram_size: usize, rut: &mut RAM) {
    for i in 0..ram_size-1 {
        rut.write(i.try_into().unwrap(),1);
        rut.read(i.try_into().unwrap());
        assert!(rut.read(i.try_into().unwrap()) == 1);
    }
    rut.reset();
}

// Clock generator
pub fn clock_gen(clock_vec: &mut Vec<u32>) {
    let mut clock = 0;
    for i in 0..CYCLES-1 {
        clock = 1;
        clock_vec.push(clock);
        clock = 0;
        clock_vec.push(clock);
    }
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
    println!("{:?}", bin_only);

    for i in 0..bin_only.len() {
        let temp_line = u32::from_str_radix(&bin_only[i], 16).unwrap();
        ram.write(i.try_into().unwrap(), temp_line);    // Error handling when overflow needs to be added here
    }
}

// stage2 -> Decode + Execute
pub fn stage2(mut proc: Vproc) {
    let mut instr: u32 = 0;

    for i in 0..PROGRAM_LEN {
        if proc.ram_module.dirty_bit[i] == 1 {
            instr = proc.ram_module.read(i.try_into().unwrap());
            let mut instr_str = format!("{:032b}", instr).to_string();
            let mut instr_str_split = instr_str.split("").collect::<Vec<_>>();
            instr_str_split.remove(0);
            instr_str_split.remove(instr_str_split.len()-1);
            let mut decoded_fields = instruction_decoder(instr_str_split, proc.clone());
            log::info!("{:?}", decoded_fields);
        }
    }
}

// Instruction Decoder
pub fn instruction_decoder(instr: Vec<&str>, mut proc: Vproc) -> Vec<i32> {
    /*
     * This decoder is based on the RISC-V Unprivleged Spec v2.2
     */

    let mut return_vec: Vec<i32> = Vec::new();  // Return vector
    if instr.len() != XLEN {
        panic!("Wrong instruction length!");
    }

    /*
     * Instruction breakdown
     * 31 --------------------------------6------0
     * 0----------------------------------25------31
     * /                                  /opcode/
     */

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
            let imm_slice = &instr[0..12];
            let imm_slice_joined = imm_slice.join("");

            match funct3_slice_joined.as_str() {
                "000" => {      // Load Byte (8-bits)
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Load Byte (LB) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LB x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "001" => {      // Load Half-word (16-bits)
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Load Half-word (LH) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LH x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "010" => {      // Load Word (32-bits)
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Load Word (LW) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LW x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());;
                    return_vec
                }
                "100" => {      // Load Byte Unsigned (u8-bits)
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Load Byte Unsigned (LBU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LBU x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "101" => {      // Load Half-word Unsigned (u16-bits)
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Load Half-word Unsigned (LHU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LHU x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                default => {
                    log::error!("Instruction format error!");
                    return_vec
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
            let imm_slice = &instr[0..12];
            let imm_slice_joined = imm_slice.join("");

            match funct3_slice_joined.as_str() {
                "000" => {      // Store Byte (8-bits)
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let rs2_bits = i32::from_str_radix(&rs2_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rs1_bits);
                    return_vec.push(rs2_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Store Byte (SB) instruction decoded");
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two address: x{}", rs2_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SB x{}, {}(x{})", rs1_bits, imm_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "001" => {      // Store Half-word (16-bit)
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let rs2_bits = i32::from_str_radix(&rs2_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rs1_bits);
                    return_vec.push(rs2_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Store Half-word (SH) instruction decoded");
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two address: x{}", rs2_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SH x{}, {}(x{})", rs1_bits, imm_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "010" => {      // Store Word (32-bit)
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let rs2_bits = i32::from_str_radix(&rs2_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rs1_bits);
                    return_vec.push(rs2_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Store Word (SW) instruction decoded");
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Register Two address: x{}", rs2_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SW x{}, {}(x{})", rs1_bits, imm_bits, rs2_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                default => {
                    log::error!("Instruction format error!");
                    return_vec
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
            let imm_slice_joined = imm_slice.join("");

            match funct3_slice_joined.as_str() {
                "000" => {      // Add immediate
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Add Immediate (ADDI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("ADDI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "010" => {      // Set less than immediate
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Set less than Immediate (SLTI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SLTI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "011" => {      // Set less than immediate unsigned
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("Set less than Immediate unsigned (SLTIU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("SLTIU x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "100" => {      // XOR Immediate
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("XOR Immediate (XORI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("XORI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "110" => {      // OR Immediate
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("OR Immediate (ORI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("ORI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                "111" => {      // AND Immediate
                    let rd_bits = i32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = i32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = i32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    thread::sleep(Duration::from_millis(250));
                    log::info!("AND Immediate (ANDI) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("ANDI x{}, x{}, {}", rd_bits, rs1_bits, imm_bits);
                    log::info!("{}", "--------------------------------".green());
                    return_vec
                }
                default => {
                    log::error!("Instruction format error!");
                    return_vec
                }
            &_ => todo!()
            }
        }
    &_ => todo!()
    }
}
