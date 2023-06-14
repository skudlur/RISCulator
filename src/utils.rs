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

// Constants
const CYCLES: u32 = 100;
const SPEED: usize = 1;
const PROGRAM_LEN: usize = 4; // replaced soon by calculating program length with a fn
const XLEN: usize = 32;

// Logo displaying function
pub fn logo_display() {
    /* RISCulator logo */
    let filename = "logo.txt";
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

// Program binary loader
pub fn program_loader(path: &str, ram: &mut RAM) {
    let bin_file = File::open(path).unwrap();
    let reader = BufReader::new(bin_file);

    let mut bin_vec = Vec::new();

    for line in reader.lines() {
        bin_vec.push(line.unwrap());
    }

    for i in 0..bin_vec.len()-1 {
        let temp_line = u32::from_str_radix(&bin_vec[i], 2).unwrap();
        ram.write(i.try_into().unwrap(), temp_line);
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

        }
    }
}

// Instruction Decoder
pub fn instruction_decoder(instr: Vec<&str>, mut proc: Vproc) -> Vec<u32> {
    /*
     * This decoder is based on the RISC-V Unprivleged Spec v2.2
     */

    let mut return_vec: Vec<u32> = Vec::new();  // Return vector
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

    log::info!("--------------------------------");

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
                    let rd_bits = u32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = u32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = u32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    log::info!("Load Byte (LB) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LB x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("--------------------------------");
                    return_vec
                }
                "001" => {      // Load Half-word (16-bits)
                    let rd_bits = u32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = u32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = u32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    log::info!("Load Half-word (LH) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LH x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("--------------------------------");
                    return_vec
                }
                "010" => {      // Load Word (32-bits)
                    let rd_bits = u32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = u32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = u32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    log::info!("Load Word (LW) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LW x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("--------------------------------");
                    return_vec
                }
                "100" => {      // Load Byte Unsigned (u8-bits)
                    let rd_bits = u32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = u32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = u32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    log::info!("Load Byte Unsigned (LBU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LBU x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("--------------------------------");
                    return_vec
                }
                "101" => {      // Load Half-word Unsigned (u16-bits)
                    let rd_bits = u32::from_str_radix(&rd_slice_joined, 2).unwrap();
                    let rs1_bits = u32::from_str_radix(&rs1_slice_joined, 2).unwrap();
                    let imm_bits = u32::from_str_radix(&imm_slice_joined, 2).unwrap();
                    return_vec.push(rd_bits);
                    return_vec.push(rs1_bits);
                    return_vec.push(imm_bits);
                    log::info!("Load Half-word Unsigned (LHU) instruction decoded");
                    log::info!("Destination Register address: x{}", rd_bits);
                    log::info!("Register One address: x{}", rs1_bits);
                    log::info!("Immediate value: {}", imm_bits);
                    log::info!("LHU x{}, {}(x{})", rd_bits, imm_bits, rs1_bits);
                    log::info!("--------------------------------");
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
            println!("{:?}", rs2_slice);
            let rs1_slice = &instr[12..17];
            let rs1_slice_joined = rs1_slice.join("");
            let imm_slice = &instr[0..12];
            let imm_slice_joined = imm_slice.join("");
            return_vec
        }
    &_ => todo!()
    }
}
