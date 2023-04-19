/* RISCulator */

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
struct Vproc {
    regs: Register,
    xlen: usize,
    ext: String,
    pc: u32,
}

// Virtual Processor (RISCulator Proc) traits
impl Vproc {
    // Initialize the Vproc object with default values
    fn new() -> Vproc {
        Vproc {
            regs: Register::new(),
            xlen: XLEN,
            ext: "I".to_string(),
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
}

// RISCulator main function
fn main() {
    let mut proc1 = Vproc {
        regs: Register::new(),
        xlen: XLEN,
        ext: EXTENSION.to_string(),
        pc: 0,
    };
    println!("{:?}", proc1);

    proc1.disp_proc_info();
    
    proc1.regs.print();
    proc1.regs.write(2,162);
    proc1.regs.print();
    proc1.reset();
    proc1.regs.print();
    
}
