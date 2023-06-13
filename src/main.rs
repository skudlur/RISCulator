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

// Utilities and other imports here
mod utils;

// Constants here (might change to yaml soon)
const EXTENSION: &str = "I";
const REG_SIZE: usize = 32;
const RAM_SIZE: usize = 1024;
const XLEN: usize = 32;
const PATH: &str = "bin.txt";
const SPEED: usize = 1;

// Register Struct
#[derive(Debug, Clone)]
pub struct Register {
    regs: [u32; REG_SIZE],
}

// Register Struct traits
impl Register {
    // Initialize the registers to 0
    fn new() -> Self {
        let regs = [0; REG_SIZE];
        Self { regs }
    }

    // Read data from an index of the register
    fn read(&mut self, index: u32) -> u32 {
        self.regs[index as usize]
    }

    // Write data to an index of the register
    fn write(&mut self, index: u32, data: u32) {
        self.regs[index as usize] = data;
    }

    // Print register data
    fn print(&mut self) {
        println!("--------------------------------"); 
        println!("Register State");
        println!("--------------------------------");
        for i in 0..REG_SIZE {
            println!("x{}: {:032b}: {}", i, self.regs[i], self.regs[i]);
        }
        println!("--------------------------------");
    }

    // Resets register state to zero
    fn reset(&mut self) {
        for i in 0..REG_SIZE {
            self.regs[i] = 0;
        }
    }
}

// RAM struct
#[derive(Debug, Clone)]
pub struct RAM {
    ram_module: [u32; RAM_SIZE],
    dirty_bit: [u32; RAM_SIZE],
}

// RAM struct impl 
impl RAM {
    // Initialize the RAM cells to 0
    fn new() -> Self {
        let ram_module = [0; RAM_SIZE];
        let dirty_bit = [0; RAM_SIZE];
        Self {
            ram_module,
            dirty_bit
        }
    }

    // Read data from an index in the main (RAM) memory
    fn read(&mut self, index: u32) -> u32 {
        self.ram_module[index as usize]
    }

    // Write data to an index of the main (RAM) memory
    fn write (&mut self, index: u32, data: u32) {
        self.ram_module[index as usize] = data;
        self.dirty_bit[index as usize] = 1;
    }

    // Print all RAM data
    fn print_all(&mut self) {
        println!("--------------------------------");
        println!("RAM");
        println!("--------------------------------");
        for i in 0..RAM_SIZE {
            println!("{}: {:032b}: {} : {}", i, self.ram_module[i], self.dirty_bit[i], self.ram_module[i]);
        }
        println!("--------------------------------");
    }

    // Print only dirty RAM data
    fn print_dirty(&mut self) {
        println!("--------------------------------");
        println!("RAM (dirty lines only)");
        println!("--------------------------------");
        for i in 0..RAM_SIZE {
            if self.dirty_bit[i] == 1 {
                println!("{}: {:032b}: {} : {}", i, self.ram_module[i], self.dirty_bit[i], self.ram_module[i]);
            }
        }
        println!("--------------------------------");
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
    pc: u32,
    mode: Mode,
    ram_module: RAM,
}

// Enumerated processor modes
#[derive(Debug, Clone)]
enum Mode {
    User,
    Supervisor,
    Machine,
}

// Virtual Processor (RISCulator Proc) traits
impl Vproc {
    // Initialize the Vproc object with default values
    fn new(regs: Register, misa: isize, pc: u32, mode: Mode, ram_module: RAM) -> Self {
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
            self.regs.write(i as u32, 0);
            self.ram_module.write(i as u32, 0);
        }
    }

    // Displays system info
    fn disp_proc_info(&self) {
        println!("--------------------------------"); 
        println!("System Information");
        println!("--------------------------------"); 
        println!("Instruction Length:");
        println!("Extensions: RV"); 
    }

    // Returns machine ISA register value
    fn get_misa(&self) -> &isize {
        &self.misa
    }

    // Returns processor mode
    fn get_mode(&self) -> &Mode {
        &self.mode
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
    let mut clock = Vec::new();
    clock.push(0);
    utils::logo_display();
    println!("|----------------- A lightweight RISC-V emulator -----------------|");
    thread::sleep(Duration::from_secs(1));
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
    thread::sleep(Duration::from_millis(10));
    let mut proc = Vproc {
        regs: Register::new(),
        misa: 4352,
        pc: 0,
        mode: Mode::User,
        ram_module: RAM::new(),
    };
    thread::sleep(Duration::from_secs(1));
    log::info!("Registers of length = {} bits initialized", XLEN);
    thread::sleep(Duration::from_millis(10));
    log::warn!("Read/write tests for Registers starting");
    proc.regs.print();
    thread::sleep(Duration::from_secs(1));
    utils::register_tests(REG_SIZE, &mut proc.regs);
    log::warn!("Register tests passed!");

    log::info!("RAM module of size = {} initialized", RAM_SIZE);
    thread::sleep(Duration::from_millis(10));
    log::warn!("Read/write tests for RAM starting");
    thread::sleep(Duration::from_secs(1));
    utils::ram_tests(RAM_SIZE, &mut proc.ram_module);
    log::warn!("RAM tests passed!");

    log::info!("Clock Generator starting at 0");
    let clock_handler = thread::spawn(move || {
        utils::clock_gen(&mut clock);
    });
    clock_handler.join().unwrap();
    log::info!("Clock Generator test complete!");
    thread::sleep(Duration::from_secs(1));
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
      └──────────────►│  FETCH  ├────────►│ Decode  ├────────►│ Execute ├────────►│ Memory  ├──────────┘
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
    ");
    thread::sleep(Duration::from_secs(1));
    log::info!("Stage 1: Fetch stage starting");
    thread::sleep(Duration::from_millis(10));
    log::info!("Prepping for fetch operations");
    utils::program_loader(PATH, &mut proc.ram_module);
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
      └──────────────►│  Fetch  ├────────►│ DECODE  ├────────►│ EXECUTE ├────────►│ Memory  ├──────────┘
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
    ");

    log::info!("Stage 2: Decode and Execute stage starting");
    utils::stage2(proc);
}
