/* RISCulator - RISC-V Emulator */
/*   Utility functions here     */

// Libraries here
use std::fs;
use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use crate::Register;
use crate::RAM;
use std::thread;
use std::time::Duration;

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
    log::info!("Loading configurations");
    log::info!("Instruction length: {}", xlen);
    log::info!("Extension: RV{}{}", xlen, extension);
    log::info!("RAM size: {}", ram_size);
}

// Register test. rut -> registers-under-test
pub fn register_tests(reg_size: usize, rut: &mut Register) {
    for i in 0..reg_size-1 {
        rut.write(i.try_into().unwrap(),1);
        rut.read(i.try_into().unwrap());
        assert!(rut.read(i.try_into().unwrap()) == 1);
    }
}

// RAM test. rut -> RAM-under-test
pub fn ram_tests(ram_size: usize, rut: &mut RAM) {
    for i in 0..ram_size-1 {
        rut.write(i.try_into().unwrap(),1);
        rut.read(i.try_into().unwrap());
        assert!(rut.read(i.try_into().unwrap()) == 1);
    }
}
