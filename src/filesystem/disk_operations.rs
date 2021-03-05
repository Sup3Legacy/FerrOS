//! Crate for every interractions with the disk

use crate::{print, println};
use x86_64::instructions::interrupts::{disable, enable};
use x86_64::instructions::port::Port;

/// Base port for the disk index 2 for QEMU
pub const DISK_PORT: u16 = 0x170;

/// Base port for the kernel in QEMU
pub const KERNEL_DISK_PORT: u16 = 0x1F0;

/// Function to test if we can read
pub fn test_read() {
    let mut a = [0_u16; 256];
    unsafe {
        read((&mut a) as *mut [u16; 256], 1, DISK_PORT);
        println!("{:x?}", a);
    }
}

/// Initialise the disk by reading it's informations (should improve it by giving an output)
pub fn init() {
    unsafe {
        disable();
        let mut data_register = Port::<u16>::new(0x170); // used to read write PIO data
        let mut sectorcount_register = Port::new(0x172);
        let mut lba_low = Port::new(0x173);
        let mut lba_mid = Port::new(0x174);
        let mut lba_high = Port::new(0x175);
        let mut drive_head_register = Port::new(0x176);
        let mut command_register = Port::new(0x177);
        drive_head_register.write(0b10100000_u8);
        sectorcount_register.write(0_u8);
        lba_low.write(0_u8);
        lba_mid.write(0_u8);
        lba_high.write(0_u8);
        //        println!("command send");

        command_register.write(0xEC_u8);
        let mut i = command_register.read();
        let mut compte = 1;
        while (i & 0x8) == 0 {
            i = command_register.read();
            compte += 1;
        }
        lba_low.read();
        lba_mid.read();
        lba_high.read();
        let mut data_table: [u16; 256] = [0; 256];
        for i in 0..256 {
            data_table[i] = data_register.read();
        }
        /*
        println!("uint16_t 0 : {}", data_table[0]);
        println!("uint16_t 83 : {} {}", data_table[83], data_table[83] & 1024);
        println!("uint16_t 88 : {}", data_table[88]);
        println!("uint16_t 93 : {}", data_table[93]);
        println!("uint32_t 61-61 : {}", (data_table[60] as u32) << 0 | ((data_table[61] as u32) << 16));
        println!("uint32_t 100-103 : {}", ((data_table[100] as u64) << 0) |
                        ((data_table[101] as u64) << 16) |
                        ((data_table[102] as u64) << 32) | ((data_table[103] as u64) << 48));
                        */
        enable(); // /!\ Should not to this if it was disabled before !
    }
}

/// function that from la sector of the disk outputs the data stored at the corresponding place (lba's count starts at 1!)
pub fn read_sector(lba: u32) -> [u16; 256] {
    let mut a = [0_u16; 256];
    unsafe {
        read((&mut a) as *mut [u16; 256], lba, DISK_PORT);
    }
    unsafe {
        //flush_cache(DISK_PORT);
    }
    a
}

unsafe fn flush_cache(port: u16) {
    disable();
    let mut command_register = Port::new(port + 7);

    command_register.write(0xE7_u8); // give the READ SECTOR command
    let mut i = command_register.read();
    let mut compte = 1;
    while (i & 0x80) != 0 {
        //println!("{} en {}", i, compte); // to warn in case of infinite loops
        i = command_register.read();
        compte += 1;
        if compte % 1000000 == 0 {
            println!("not finished 1-1 : {} en {}", i, compte); // to warn in case of infinite loops
        }
    }
    enable();
}

/// Read function that reads in any disk if the right port is given.
unsafe fn read(table: *mut [u16; 256], lba: u32, port: u16) {
    disable();
    println!("Reading from sector {}", lba);
    let lba = lba as u64;
    let mut data_register = Port::<u16>::new(port + 0);
    let mut sectorcount_register = Port::new(port + 2);
    let mut lba_low = Port::new(port + 3);
    let mut lba_mid = Port::new(port + 4);
    let mut lba_high = Port::new(port + 5);
    let mut drive_head_register = Port::new(port + 6);
    let mut command_register = Port::new(port + 7);
    drive_head_register.write(0x40_u32); // outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F))
    sectorcount_register.write(0_u8); // says that we want to read only one register
    lba_low.write((lba >> 24) as u8);
    lba_mid.write((lba >> 32) as u8);
    lba_high.write((lba >> 40) as u8);
    sectorcount_register.write(1_u8); // says that we want to read only one register
    lba_low.write(lba as u8);
    lba_mid.write((lba >> 8) as u8);
    lba_high.write((lba >> 16) as u8);
    command_register.write(0x24_u8); // give the READ SECTOR command

    // waits for the disk to be ready for transfer
    let mut i = command_register.read();
    let mut compte = 1;
    while (i & 0x80) != 0 || (i & 0b00001000_u8) == 0 {
        i = command_register.read();
        compte += 1;
        if compte % 1000000 == 0 {
            println!("not finished 1 : {} en {}", i, compte); // to warn in case of infinite loops
        }
    }

    for i in 0..256 {
        let t = data_register.read(); // reads all the data one by one. The loop is mandatory to give the drive the time to give the data
        (*table)[i] = t;
    }

    let mut i = command_register.read();
    let mut compte = 1;
    while (i & 0x80) != 0 {
        i = command_register.read();
        compte += 1;
        if compte % 1000000 == 0 {
            println!("not finished 2 : {} en {}", i, compte); // to warn in case of infinite loops
        }
    }
    enable(); // /!\ Should not to this if it was disabled before !
}

/// function that from la sector of the disk writes the given data at the corresponding place (lba's count starts at 1!)
pub fn write_sector(table: &[u16; 256], lba: u32) {
    unsafe {
        write(table, lba, DISK_PORT);
    }
    unsafe {
        flush_cache(DISK_PORT);
    }
}

/// Write function that writes in any disk if the right port is given.
unsafe fn write(table: &[u16; 256], lba: u32, port: u16) {
    disable();
    let lba = lba as u64;
    let mut data_register = Port::<u16>::new(port + 0);
    let mut sectorcount_register = Port::new(port + 2);
    let mut lba_low = Port::new(port + 3);
    let mut lba_mid = Port::new(port + 4);
    let mut lba_high = Port::new(port + 5);
    let mut drive_head_register = Port::new(port + 6);
    let mut command_register = Port::new(port + 7);
    drive_head_register.write(0x40_u32); // outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F))
    sectorcount_register.write(0_u8); // says that we want to write only one register
    lba_low.write((lba >> 24) as u8);
    lba_mid.write((lba >> 32) as u8);
    lba_high.write((lba >> 40) as u8);
    sectorcount_register.write(1_u8); // says that we want to read only one register
    lba_low.write(lba as u8);
    lba_mid.write((lba >> 8) as u8);
    lba_high.write((lba >> 16) as u8);
    command_register.write(0x34_u8); // give the READ SECTOR command

    let mut i = command_register.read();
    let mut compte = 1;
    while (i & 0x80) != 0 || (i & 0b00001000_u8) == 0 {
        i = command_register.read();
        compte += 1;
        if compte % 1000000 == 0 {
            println!("not finished 3 : {} en {}", i, compte); // to warn in case of infinite loops
        }
    }
    let mut delay = Port::new(0x80);
    for i in 0..256 {
        data_register.write(table[i]); // writes all the data one by one. The loop is mandatory to give the drive the time to accept the data
        delay.write(0_u8);
    }

    command_register.write(0xE7);

    let mut i = command_register.read();
    let mut compte = 1;
    while (i & 0x80) != 0 {
        i = command_register.read();
        compte += 1;
        if compte % 1000000 == 0 {
            println!("not finished 3 : {} en {}", i, compte); // to warn in case of infinite loops
        }
    }
    enable(); // /!\ Should not to this if it was disabled before !
}
