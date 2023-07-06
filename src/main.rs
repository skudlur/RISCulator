/* RISCulator - RISC-V Emulator */
/*         Main file            */
#![allow(warnings, unused)]

// Libraries here
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::thread;
use std::time::Duration;
use std::thread::spawn;
use std::fmt::Binary;
use colored::*;
use std::mem::MaybeUninit;

// Utilities and other imports here
mod utils;

// Constants here (might change to yaml soon)
const EXTENSION: &str = "I";
const REG_SIZE: usize = 32;
const RAM_SIZE: usize = 100;
const XLEN: usize = 32;
const PATH: &str = "bin.txt";
const SPEED: usize = 1;
const INI: isize = 4; // Init address offset

// Static
static mut RAM_SIZE_NEW: usize = RAM_SIZE;

// Register Struct
#[derive(Debug, Clone, Copy)]
pub struct Register {
    regs: [isize; REG_SIZE],
    dirty_bit: [u32; REG_SIZE],
}

// Register Struct traits
impl Register {
    // Initialize the registers to 0
    fn new() -> Self {
        let regs = [0; REG_SIZE];
        let dirty_bit = [0; REG_SIZE];
        Self {
            regs,
            dirty_bit
        }
    }

    // Read data from an index of the register
    fn read(&mut self, index: usize) -> isize {
        self.regs[index]
    }

    // Write data to an index of the register
    fn write(&mut self, index: usize, data: isize) {
        self.regs[index] = data;
        self.dirty_bit[index] = 1;
    }

    // Print register data
    fn print(&mut self) {
        println!("{}", "--------------------------------".green());
        println!("{}", "Register State".green());
        println!("{}", "--------------------------------".green());
        for i in 0..REG_SIZE {
            println!("x{}: {:032b}: {} : {}", i, self.regs[i], self.dirty_bit[i], self.regs[i]);
        }
        println!("{}", "--------------------------------".green());
    }

    // Print only dirty RAM data
    fn print_dirty(&mut self) {
        println!("{}", "--------------------------------".green());
        println!("{}", "Register (dirty lines only)".green());
        println!("{}", "--------------------------------".green());
        for i in 0..REG_SIZE {
            if self.dirty_bit[i] == 1 {
                println!("x{}: {:032b}: {} : {:08x}", i, self.regs[i], self.dirty_bit[i], self.regs[i]);
            }
        }
        println!("{}", "--------------------------------".green());
    }

    // Resets register state to zero
    fn reset(&mut self) {
        for i in 0..REG_SIZE {
            self.regs[i] = 0;
            self.dirty_bit[i] = 0;
        }
    }
}


// RAM struct
#[derive(Debug, Clone)]
pub struct RAM {
    address: Vec<usize>,
    ram_module: Vec<isize>,
    dirty_bit: Vec<u32>,
}

// RAM struct impl 
impl RAM {
    // Initialize the RAM cells to 0
    fn new() -> Self {
        let mut address = vec![0; RAM_SIZE];
        let mut ram_module = vec![0; RAM_SIZE];
        let mut dirty_bit = vec![0; RAM_SIZE];
        Self {
            address,
            ram_module,
            dirty_bit
        }
    }

    // Read data from an index in the main (RAM) memory
    fn read(&mut self, index: usize) -> isize {
        self.ram_module[index]
    }

    // Write data to an index of the main (RAM) memory
    fn write(&mut self, index: usize, data: isize) {
        self.ram_module[index] = data;
        self.dirty_bit[index] = 1;
    }

    // Write to new address (RAM emulation)
    fn write_to_addr(&mut self, index_addr: usize, data: isize) {
        for i in 0..RAM_SIZE {
            if self.dirty_bit[i] == 0 {
                self.address[i] = index_addr;
                self.ram_module[i] = data;
                self.dirty_bit[i] = 1;
                break;
            }
        }
    }

    // Read from address (RAM emulation)


    // Print all RAM data
    fn print_all(&mut self) {
        println!("{}", "--------------------------------".green());
        println!("{}", "RAM".green());
        println!("{}", "--------------------------------".green());
        for iter in 0..unsafe{RAM_SIZE_NEW} {
            println!("{:#010x}: {:032b}: {} : {}", self.address[iter], self.ram_module[iter], self.dirty_bit[iter], self.ram_module[iter]);
        }
        println!("{}", "--------------------------------".green());
    }

    // Print only dirty RAM data
    fn print_dirty(&mut self) {
        println!("{}", "--------------------------------".green());
        println!("{}", "RAM (dirty lines only)".green());
        println!("{}", "--------------------------------".green());
        for iter in 0..unsafe{RAM_SIZE_NEW} {
            if self.dirty_bit[iter] == 1 {
                println!("{:#010x}: {:032b}: {} : {:08x}", self.address[iter], self.ram_module[iter], self.dirty_bit[iter], self.ram_module[iter]);
            }
            else {
                break;
            }
        }
        println!("{}", "--------------------------------".green());
    }

    // Reset RAM to zero
    fn reset(&mut self) {
        for i in 0..RAM_SIZE {
            self.ram_module[i] = 0;
            self.dirty_bit[i] = 0;
        }
    }
}

// Virtual Processor (RISCulator Proc) Struct
#[derive(Debug, Clone)]
pub struct Vproc {
    regs: Register,
    misa: isize,
    pc: isize,
    mode: Mode,
    ram_module: RAM,
}

// Enumerated processor modes
#[derive(Debug, Clone, Copy)]
enum Mode {
    User,
    Supervisor,
    Machine,
}

// Virtual Processor (RISCulator Proc) traits
impl Vproc {
    // Initialize the Vproc object with default values
    fn new(regs: Register, misa: isize, pc: isize, mode: Mode, ram_module: RAM) -> Self {
        Vproc {
            regs,
            misa,
            pc,
            mode,
            ram_module,
        }
    }

    // Resets the Vproc
    fn reset(&mut self) {
        self.pc = 0;
        for i in 0..REG_SIZE-1 {
            self.regs.write(i.try_into().unwrap(), 0);
            self.ram_module.write(i.try_into().unwrap(), 0);
        }
    }

    // Updates registers
    fn update_regs(&mut self, mut new_regs: Register) {
        for i in 0..REG_SIZE {
            if new_regs.dirty_bit[i] == 1 {
                let new_regs_up_line = new_regs.read(i.try_into().unwrap());
                self.regs.write(i.try_into().unwrap(), new_regs_up_line);
            }
        }
    }

    // Increment PC
    fn pc_incr(&mut self, value: isize) {
        self.pc += value;
    }

    // Updates RAM
    fn update_ram(&mut self, mut new_ram: RAM) {
        let mut iter = 0;
        loop {
            if new_ram.dirty_bit[iter] == 1 && iter < unsafe{RAM_SIZE_NEW} {
                let new_ram_up_line = new_ram.read(iter.try_into().unwrap());
                let new_address = new_ram.address[iter];
                for i in 0..unsafe{RAM_SIZE_NEW} {
                    if self.ram_module.dirty_bit[i] == 0  {
                        self.ram_module.address[i] = new_address;
                        self.ram_module.write(i.try_into().unwrap(), new_ram_up_line);
                        unsafe{RAM_SIZE_NEW += 1};
                        break;
                    }
                    else {
                        continue;
                    }
                }
                break;
            }
            iter += 1;
        }
    }

    // misa breakdown and process
    fn misa_slice(&self) -> String {
        let temp_misa = self.misa.to_le();
        let temp_misa_bin = format!("{:032b}", temp_misa);
        let mut slice_misa = temp_misa_bin.to_string().chars().collect::<Vec<_>>();
        slice_misa.reverse();
        let mut misa_ext = Vec::new();
        for i in 0..25 {
            let temp_ext_slice = &slice_misa[i];
            misa_ext.push(temp_ext_slice);
        }
        let misa_ext_con: String = misa_ext.clone().into_iter().collect(); // Little endian Extension slice
        let misa_ext_be: String = misa_ext_con.clone().chars().rev().collect(); // Big endian
        println!("{}",misa_ext_be);                                                                            
        let misa_ext_be_vec: Vec<char> = misa_ext_be.chars().collect();
        
        let mut ext_vec: Vec<char> = vec![]; // Extension vector
        
        // Extension check
        if misa_ext_be_vec[24] == '1' { // Atomic Extension 
            ext_vec.push('A');
        }
        if misa_ext_be_vec[23] == '1' { // Bit manip extensions
            ext_vec.push('B');
        }
        if misa_ext_be_vec[22] == '1' { // Compressed extensions
            ext_vec.push('C');
        }
        if misa_ext_be_vec[21] == '1' { // Double-precision FP extension
            ext_vec.push('D');
        }
        if misa_ext_be_vec[20] == '1' { // Embedded extension
            ext_vec.push('E');
        }
        if misa_ext_be_vec[19] == '1' { // Single-precision FP extension
            ext_vec.push('F');
        }
        if misa_ext_be_vec[16] == '1' { // Integer extension
            ext_vec.push('I');
        }
        if misa_ext_be_vec[12] == '1' { // Multiply extension
            ext_vec.push('M');
        }
        if misa_ext_be_vec[9] == '1' { // Packed SIMD extension
            ext_vec.push('P');
        }

        println!("{:?}", ext_vec);
        let ext_vec_rec: String = ext_vec.iter().collect();
        ext_vec_rec
    }
}

// RISCulator main function
fn main() {
    let mut path: String = "test/main.c".to_string();
    utils::logo_display();
    println!("{}", "|----------------- A lightweight RISC-V emulator -----------------|".red());
    utils::boot_seq(XLEN, EXTENSION, REG_SIZE, RAM_SIZE);

    // Logging
    Builder::new()
            .format(|buf, record| {
                writeln!(buf,
                    "{} [{}] - {}",
                    Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .filter(None, LevelFilter::Info);

    log::info!("Creating a virtual processor with the given configuration");
    let mut proc = Vproc {
        regs: Register::new(),
        misa: 4352,
        pc: 0,
        mode: Mode::User,
        ram_module: RAM::new(),
    };
    log::info!("Registers of length = {} bits initialized", XLEN);
    log::warn!("Read/write test>s for Registers starting");
    proc.regs.print();;
    utils::register_tests(REG_SIZE, &mut proc.regs);
    log::warn!("Register tests passed!");
    log::info!("RAM module of size = {} initialized", RAM_SIZE);
    log::warn!("Read/write tests for RAM starting");
    utils::ram_tests(RAM_SIZE, &mut proc.ram_module);
    log::warn!("RAM tests passed!");
    println!("
                                         RISCulator emulation stages

      ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
      │                                                                                                │
      ▼                                                                                                │
┌───────────┐                                                                                          │
│           │                                                                                          │
│           │                                                                                          │
│  Memory   │         ┌─────────┐         ┌─────────┐         ┌─────────┐         ┌─────────┐          │
│           │         │         │         │         │         │         │         │         │          │
│           │         │         │         │         │         │         │         │         │          │
│           │         │         │         │         │         │         │         │         │          │
└─────┬─────┘         │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      └──────────────►│   {} ├────────►│ Decode  ├────────►│ Execute ├────────►│ Memory  ├──────────┘
                      │         │         │         │         │         │         │ Access  │ Writeback
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      └─────────┘         └─────────┘         └─────────┘         └─────────┘
    ", "Fetch".green());
    log::info!("Stage 1: Fetch stage starting");
    log::info!("Prepping for fetch operations");
    let program_parsed = utils::program_parser("test/out.txt", &mut proc.ram_module);
    log::info!("Program loaded to main memory!");
    proc.ram_module.print_dirty();
    println!("
                                         RISCulator emulation stages

      ┌────────────────────────────────────────────────────────────────────────────────────────────────┐
      │                                                                                                │
      ▼                                                                                                │
┌───────────┐                                                                                          │
│           │                                                                                          │
│           │                                                                                          │
│  Memory   │         ┌─────────┐         ┌─────────┐         ┌─────────┐         ┌─────────┐          │
│           │         │         │         │         │         │         │         │         │          │
│           │         │         │         │         │         │         │         │         │          │
│           │         │         │         │         │         │         │         │         │          │
└─────┬─────┘         │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      │               │         │         │         │         │         │         │         │          │
      └──────────────►│  Fetch  ├────────►│  {} ├────────►│ {} ├────────►│ Memory  ├──────────┘
                      │         │         │         │         │         │         │ Access  │ Writeback
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      │         │         │         │         │         │         │         │
                      └─────────┘         └─────────┘         └─────────┘         └─────────┘
    ", "Decode".green(), "Execute".green());
    log::info!("Stage 2: Decode and Execute stage starting");
    utils::stage2(&mut proc);
    proc.regs.print_dirty();
}
