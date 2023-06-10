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
use std::thread;
use std::time::Duration;
use std::thread::spawn;

// Constants
const CYCLES: u32 = 100;

// Logo displaying function
pub fn logo_display() {
    /* RISCulator logo */
    let filename = "logo.txt";
    let logo_con = fs::read_to_string(filename)
        .expect("Failed to read the file");
    println!("{}",logo_con);
}

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
        ram.write(i.try_into().unwrap(), bin_vec[i].parse::<u32>().unwrap().try_into().unwrap());
    }
}
