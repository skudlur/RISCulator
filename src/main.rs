/* RISCulator */

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

    // Read data from register
    fn read(&mut self, index: u32) -> u32 {
        self.regs[index as usize]
    }

    // Write data to register
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

// RISCulator main function
fn main() {
    let mut reg1 = Register::default();
    println!("{:?}", reg1);

    reg1.write(2,162);
    println!("{:?}", reg1);

    let temp = reg1.read(2);
    let temp = format!("{:032b}",temp);
    println!("{}", temp);

    reg1.print();
}
