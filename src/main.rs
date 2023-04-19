/* RISCulator */

// Register Struct
#[derive(Default,Debug)]
struct Register {
    regs: [u32; 32],
}

// Register Struct traits
impl Register {
    // Initialize the registers to 0
    fn new() -> Self {
        let regs = [0; 32];
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
}

// RISCulator main function
fn main() {
    let mut reg1 = Register::default();
    println!("{:?}", reg1);

    reg1.write(2,162);
    println!("{:?}", reg1);

    let temp = reg1.read(2);
    println!("{}", temp); 
}
