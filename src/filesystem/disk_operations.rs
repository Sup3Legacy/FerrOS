use crate::{print, println};
use x86_64::instructions::interrupts::{disable, enable};
use x86_64::instructions::port::Port;

pub const DISK_PORT: u16 = 0x170;

pub fn test_read() {
    let mut a = [0 as u16; 256];
    unsafe {
        read((&mut a) as *mut [u16; 256], 1, DISK_PORT);
        println!("{:x?}", a);
    }
}

pub fn init() {
    unsafe {
        disable();
        // println!("disque 2");
        let mut master_drive = Port::new(0x176);
        let mut control_port_base = Port::<u8>::new(0x377);
        // println!("control port base : {}", control_port_base.read());
        let mut sectorcount = Port::new(0x172);
        let mut _LBAlo = Port::new(0x173);
        let mut _LBAmid = Port::new(0x174);
        let mut _LBAhi = Port::new(0x175);
        let mut commandPort = Port::new(0x177);
        disable();
        master_drive.write(0b10100000 as u8);
        sectorcount.write(0 as u8);
        _LBAlo.write(0 as u8);
        _LBAmid.write(0 as u8);
        _LBAhi.write(0 as u8);
        //        println!("command send");

        commandPort.write(0xEC as u8);
        let mut i = commandPort.read();
        let mut compte = 1;
        while (i & 0x8) == 0 && (i & 1) == 0 {
            i = commandPort.read();
            compte = 1 + compte;
        }
        _LBAlo.read();
        _LBAmid.read();
        _LBAhi.read();
        let mut next_port = Port::<u16>::new(0x170);
        let mut table: [u16; 512] = [0; 512];
        for i in 0..256 {
            table[i] = next_port.read();
        }
        enable();
    }
}

pub fn read_sector(lba: u32) -> [u16; 256] {
    let mut a = [0 as u16; 256];
    unsafe {
        read((&mut a) as *mut [u16; 256], lba, DISK_PORT);
    }
    a
}

unsafe fn read(table: *mut [u16; 256], lba: u32, port: u16) {
    disable();
    let mut master_drive = Port::new(port + 6);
    let mut sectorcount = Port::new(port + 2);
    let mut _LBAlo = Port::new(port + 3);
    let mut _LBAmid = Port::new(port + 4);
    let mut _LBAhi = Port::new(port + 5);
    let mut commandPort = Port::new(port + 7);
    master_drive.write(0xE0 | ((lba >> 24) & 0x0F)); // outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F))
    sectorcount.write(1 as u8);
    _LBAlo.write(lba as u8);
    _LBAmid.write((lba >> 8) as u8);
    _LBAhi.write((lba >> 16) as u8);
    commandPort.write(0x20 as u8);
    let mut i = commandPort.read();
    let mut compte = 1;
    while (i & 0x80) != 0 {
        i = commandPort.read();
        compte = 1 + compte;
        if compte % 1000000 == 0 {
            println!("not finished 1 : {} en {}", i, compte);
        }
    }
    //println!("finished : {} en {}", i, compte);
    _LBAlo.read();
    _LBAmid.read();
    _LBAhi.read();
    let mut next_port = Port::<u16>::new(port + 0);
    for i in 0..256 {
        let t = next_port.read();
        // print!(" {:x?}", t);
        (*table)[i] = t;
    }
    let mut i = commandPort.read();
    let mut compte = 1;
    while (i & 0x80) != 0 {
        i = commandPort.read();
        compte = 1 + compte;
        if compte % 1000000 == 0 {
            println!("not finished 2 : {} en {}", i, compte);
        }
    }
    enable();
}

pub fn write_sector(table: &[u16; 256], lba: u32) -> () {
    unsafe {
        write(table, lba, DISK_PORT);
    }
}

unsafe fn write(table: &[u16; 256], lba: u32, port: u16) {
    disable();
    let mut master_drive = Port::new(port + 6);
    let mut sectorcount = Port::new(port + 2);
    let mut _LBAlo = Port::new(port + 3);
    let mut _LBAmid = Port::new(port + 4);
    let mut _LBAhi = Port::new(port + 5);
    let mut commandPort = Port::new(port + 7);
    master_drive.write(0xE0 | ((lba >> 24) & 0x0F)); // outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F))
    sectorcount.write(1 as u8);
    _LBAlo.write(lba as u8);
    _LBAmid.write((lba >> 8) as u8);
    _LBAhi.write((lba >> 16) as u8);
    commandPort.write(0x30 as u8);
    let mut i = commandPort.read();
    let mut compte = 1;
    while (i & 0x80) != 0 {
        i = commandPort.read();
        compte = 1 + compte;
        if compte % 1000000 == 0 {
            //println!("not finished : {} en {}", i, compte);
        }
    }
    //println!("finished : {} en {}", i, compte);
    _LBAlo.read();
    _LBAmid.read();
    _LBAhi.read();
    let mut next_port = Port::<u16>::new(port + 0);
    for i in 0..256 {
        next_port.write(table[i]);
    }
    let mut i = commandPort.read();
    let mut compte = 1;
    while (i & 0x80) != 0 {
        i = commandPort.read();
        compte = 1 + compte;
        if compte % 1000000 == 0 {
            println!("not finished 2 : {} en {}", i, compte);
        }
    }
    enable();
}
