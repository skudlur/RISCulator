/* RISCulator - RISC-V Emulator*/
use std::fs::File;
use std::io::{BufRead, BufReader};

const EXTENSION: &str = "I";
const REG_SIZE: usize = 32;
const XLEN: usize = 32;

// Register Struct
#[derive(Debug)]
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
        for i in 0..REG_SIZE {
            println!("x{}: {:032b}: {}", i, self.regs[i], self.regs[i])
        }
        println!("--------------------------------");
    }
}

// Virtual Processor (RISCulator Proc) Struct
#[derive(Debug)]
struct Vproc {
    regs: Register,
    misa: isize,
    pc: u32,
    mode: Mode,
}

// Enumerated processor modes
#[derive(Debug)]
enum Mode {
    User,
    Supervisor,
    Machine,
}

// Virtual Processor (RISCulator Proc) traits
impl Vproc {
    // Initialize the Vproc object with default values
    fn new(regs: Register, misa: isize, pc: u32, mode: Mode) -> Self {
        Vproc {
            regs,
            misa,
            pc,
            mode,
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
    fn misa_slice(&self) {
        let temp_misa = self.misa.to_le();
        let temp_misa_bin = format!("{:032b}", temp_misa);
        let mut slice_misa = temp_misa_bin.to_string().chars().collect::<Vec<_>>();
        slice_misa.reverse();
        let mut misa_ext = Vec::new();
        for i in 0..25 {
            let temp_ext_slice = &slice_misa[i];
            misa_ext.push(temp_ext_slice);
        }
        let misa_ext_con: String = misa_ext.clone().into_iter().collect();
        println!("{:?}", misa_ext_con);
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
        misa: 1,
        pc: 0,
        mode: Mode::User,
    };
    proc1.disp_proc_info();
    proc1.regs.print();

    let misa_temp = proc1.get_misa();
    println!("{:032b}", misa_temp);

    let mode_temp = proc1.get_mode();
    println!("{:?}", mode_temp);

    proc1.misa_slice();
}
