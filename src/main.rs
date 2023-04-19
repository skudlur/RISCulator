/* RISCulator - RISC-V Emulator*/
use std::fs::File;
use std::io::{BufRead, BufReader};

const EXTENSION: &str = "I";
const REG_SIZE: usize = 32;
const XLEN: usize = 32;

// Register Struct
#[derive(Default,Debug)]
struct Register {
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
        for i in 0..REG_SIZE-1 {
            println!("x{}: {:032b}: {}", i, self.regs[i], self.regs[i])
        }
        println!("--------------------------------");
    }
}

// Virtual Processor (RISCulator Proc) Struct
#[derive(Default,Debug)]
struct Vproc<'a> {
    regs: Register,
    xlen: usize,
    ext: &'a str,
    pc: u32,
}

// Virtual Processor (RISCulator Proc) traits
impl Vproc<'_> {
    // Initialize the Vproc object with default values
    fn new() -> Vproc<'static> {
        Vproc {
            regs: Register::new(),
            xlen: XLEN,
            ext: EXTENSION,
            pc: 0,
        }
    }
    
    // Resets the Vproc
    fn reset(&mut self) {
        self.pc = 0;
        for i in 0..REG_SIZE-1 {
            self.regs.write(i as u32, 0);
        }
    }

    // Displays system info
    fn disp_proc_info(&mut self) {
        println!("--------------------------------"); 
        println!("System Information");
        println!("--------------------------------"); 
        println!("Instruction Length: {}", self.xlen);
        println!("Extensions: RV{}{}", self.xlen, self.ext); 
    }
/*
    fn execute(&mut self) {
        let bin_file = File::open("bin.txt").unwrap();
        let reader = BufReader::new(bin_file);

        let mut bin_vec = Vec::new();

        for line in reader.lines() {
            bin_vec.push(line.unwrap());
        }

        match self.ext {
           "I" => {
               for i in 0..bin_vec.len() {
                   let mut curr_instr: String = bin_vec[i];
                   let mut curr_instr_vec = Vec::new();
                   let curr_instr_vec: Vec<&str> = curr_instr.split("").collect();
                   if curr_instr_vec.len() < 32 { panic!("Invalid instruction length!"); }
               }
           }
           _ => {
               panic!("Extension Error!");
           }
       }
    }
*/
}

// RISCulator main function
fn main() {
    let bin_file = File::open("bin.txt").unwrap();
    let reader = BufReader::new(bin_file);

    let mut bin_vec = Vec::new();

    for line in reader.lines() {
        bin_vec.push(line.unwrap());
    }

    println!("{:?}", bin_vec);

    let mut proc1 = Vproc {
        regs: Register::new(),
        xlen: XLEN,
        ext: EXTENSION,
        pc: 0,
    };
    proc1.disp_proc_info();
    proc1.regs.print();    
}
