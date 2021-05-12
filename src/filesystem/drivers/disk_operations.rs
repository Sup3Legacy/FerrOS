//! Crate for every interractions with the disk

use crate::println;

use x86_64::instructions::port::Port;

/// Base port for the kernel in QEMU
pub const KERNEL_DISK_PORT: u16 = 0x1F0;

/// Function to test if we can read
pub fn test_read(port: u16) {
    let mut a = [0_u16; 256];
    unsafe {
        read((&mut a) as *mut [u16; 256], 1, port);
        println!("{:x?}", a);
    }
}

/// Initialise the disk by reading it's informations (should improve it by giving an output)
pub fn init(port: u16) {
    unsafe {
        // disable();
        let mut data_register = Port::<u16>::new(port); // used to read write PIO data
        let mut sectorcount_register = Port::new(port + 2);
        let mut lba_low = Port::new(port + 3);
        let mut lba_mid = Port::new(port + 4);
        let mut lba_high = Port::new(port + 5);
        let mut drive_head_register = Port::new(port + 6);
        let mut command_register = Port::new(port + 7);
        drive_head_register.write(0b10100000_u8);
        sectorcount_register.write(0_u8);
        lba_low.write(0_u8);
        lba_mid.write(0_u8);
        lba_high.write(0_u8);
        //        println!("command send");

        command_register.write(0xEC_u8);

        let mut i = command_register.read();
        let mut _compte = 1; // unused variable?
        while (i & 0x8) == 0 {
            i = command_register.read();
            _compte += 1;
        }
        lba_low.read();
        lba_mid.read();
        lba_high.read();
        let mut data_table: [u16; 256] = [0; 256];
        for elt in &mut data_table {
            *elt = data_register.read();
        }

        println!("uint16_t 0 : {}", data_table[0]);
        println!("uint16_t 83 : {} {}", data_table[83], data_table[83] & 1024);
        println!("uint16_t 88 : {}", data_table[88]);
        println!("uint16_t 93 : {}", data_table[93]);
        println!(
            "uint32_t 61-61 : {}",
            (data_table[60] as u32) | ((data_table[61] as u32) << 16)
        );
        println!(
            "uint32_t 100-103 : {}",
            (data_table[100] as u64)
                | ((data_table[101] as u64) << 16)
                | ((data_table[102] as u64) << 32)
                | ((data_table[103] as u64) << 48)
        );

        wait_bsy(port);
        //  enable(); // /!\ Should not to this if it was disabled before !
    }
}

/// function that from la sector of the disk outputs the data stored at the corresponding place (lba's count starts at 1!)
pub fn read_sector(lba: u32, port: u16) -> [u16; 256] {
    //println!("read_sector : {} {}", lba, port);
    let mut a = [0_u16; 256];
    unsafe {
        read((&mut a) as *mut [u16; 256], lba, port);
    }
    unsafe {
        flush_cache(port);
    }
    //println!("flushed");
    a
}

/// Read function that reads in any disk if the right port is given.
unsafe fn read(table: *mut [u16; 256], lba: u32, port: u16) {
    //disable();

    wait_bsy(port);

    //println!("Reading from sector {}", lba);
    let lba = lba as u64;

    let mut data_register = Port::<u16>::new(port);
    let mut sectorcount_register = Port::new(port + 2);
    let mut lba_low = Port::new(port + 3);
    let mut lba_mid = Port::new(port + 4);
    let mut lba_high = Port::new(port + 5);
    let mut drive_head_register = Port::new(port + 6);
    let mut command_register = Port::new(port + 7);

    drive_head_register.write(0x40_u8); // outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F))
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
    wait_bsy(port);
    wait_drq(port);

    for i in 0..256 {
        let t = data_register.read(); // reads all the data one by one. The loop is mandatory to give the drive the time to give the data
        (*table)[i] = t;
    }

    wait_bsy(port);

    //enable(); // /!\ Should not to this if it was disabled before !
}

/// function that from la sector of the disk writes the given data at the corresponding place (lba's count starts at 1!)
pub fn write_sector(table: &[u16; 256], lba: u32, port: u16) {
    unsafe {
        write(table, lba, port);
    }
    unsafe {
        flush_cache(port);
    }
}

/// Writes the array table to the given sector (`lba`) at disk given by `port`.
unsafe fn write(table: &[u16; 256], lba: u32, port: u16) {
    //disable();

    wait_bsy(port);

    let lba = lba as u64;

    let mut data_register = Port::<u16>::new(port);
    let mut sectorcount_register = Port::new(port + 2);
    let mut lba_low = Port::new(port + 3);
    let mut lba_mid = Port::new(port + 4);
    let mut lba_high = Port::new(port + 5);
    let mut drive_head_register = Port::new(port + 6);
    let mut command_register = Port::new(port + 7);

    drive_head_register.write(0x40_u8); // outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F))
    sectorcount_register.write(0_u8); // says that we want to write only one register
    lba_low.write((lba >> 24) as u8);
    lba_mid.write((lba >> 32) as u8);
    lba_high.write((lba >> 40) as u8);
    sectorcount_register.write(1_u8); // says that we want to read only one register
    lba_low.write(lba as u8);
    lba_mid.write((lba >> 8) as u8);
    lba_high.write((lba >> 16) as u8);
    command_register.write(0x34_u8); // give the READ SECTOR command

    wait_bsy(port);
    wait_drq(port);

    let mut delay = Port::new(0x80);
    for elt in table.iter().take(256) {
        data_register.write(*elt); // writes all the data one by one. The loop is mandatory to give the drive the time to accept the data
        delay.write(0_u8);
    }

    command_register.write(0xE7);

    wait_bsy(port);

    //enable(); // /!\ Should not to this if it was disabled before !
}

unsafe fn wait_bsy(port: u16) {
    let mut port: Port<u16> = Port::new(port + 7);
    while port.read() & 0x80 != 0 {}
}

unsafe fn wait_drq(port: u16) {
    let mut port: Port<u16> = Port::new(port + 7);
    while port.read() & 0x08 == 0 {}
}

unsafe fn flush_cache(port: u16) {
    let mut device_head_register = Port::new(port + 6);
    let mut command_register = Port::new(port + 7);
    device_head_register.write(0x40_u8);
    command_register.write(0xE7_u8); // give the READ SECTOR command

    let mut status = command_register.read();
    while ((status & 0x80) == 0x80) && ((status & 0x01) != 0x01) {
        status = command_register.read();
    }
}
