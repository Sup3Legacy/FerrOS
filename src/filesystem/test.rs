use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts::{disable, enable};
use crate::{print, println};

pub fn testOld() {
    unsafe {
        let mut master_drive = Port::new(0x1F6);
        let mut control_port_base = Port::<u8>::new(0x3F7);
        println!("control port base : {}", control_port_base.read());
        let mut sectorcount = Port::new(0x1F2);
        let mut _LBAlo = Port::new(0x1F3);
        let mut _LBAmid = Port::new(0x1F4);
        let mut _LBAhi = Port::new(0x1F5);
        let mut commandPort = Port::new(0x1F7);
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
            if compte % 100 == 0 {
                println!("finished : {} en {}", i, compte);
            }
        }
        println!("finished : {} en {}", i, compte);
        println!("status : {}, {}, {}", _LBAlo.read(), _LBAmid.read(), _LBAhi.read());
        let mut next_port = Port::<u16>::new(0x1F0);
        let mut table:[u16; 512] = [0; 512];
        for i in 0..256 {
            table[i] = next_port.read();
        }

        println!("uint16_t 0 : {}", table[0]);
        println!("uint16_t 83 : {} {}", table[83], table[83] & 1024);
        println!("uint16_t 88 : {}", table[88]);
        println!("uint16_t 93 : {}", table[93]);
        println!("uint32_t 61-61 : {}", (table[60] as u32) << 0 | ((table[61] as u32) << 16));
        println!("uint32_t 100-103 : {}", ((table[100] as u64) << 0) |
                        ((table[101] as u64) << 16) |
                        ((table[102] as u64) << 32) | ((table[103] as u64) << 48));

        /* test disque dur secondaire */
        println!("disque 2");
        let mut master_drive = Port::new(0x176);
        let mut control_port_base = Port::<u8>::new(0x377);
        println!("control port base : {}", control_port_base.read());
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
            if compte % 100 == 0 {
                println!("finished : {} en {}", i, compte);
            }
        }
        println!("finished : {} en {}", i, compte);
        println!("status : {}, {}, {}", _LBAlo.read(), _LBAmid.read(), _LBAhi.read());
        let mut next_port = Port::<u16>::new(0x170);
        let mut table:[u16; 512] = [0; 512];
        for i in 0..256 {
            table[i] = next_port.read();
        }

        println!("uint16_t 0 : {}", table[0]);
        println!("uint16_t 83 : {} {}", table[83], table[83] & 1024);
        println!("uint16_t 88 : {}", table[88]);
        println!("uint16_t 93 : {}", table[93]);
        println!("uint32_t 61-61 : {}", (table[60] as u32) << 0 | ((table[61] as u32) << 16));
        println!("uint32_t 100-103 : {}", ((table[100] as u64) << 0) |
                        ((table[101] as u64) << 16) |
                        ((table[102] as u64) << 32) | ((table[103] as u64) << 48));
                
                
        enable();
    }
}


pub fn test() {
    unsafe {
        disable();
        println!("disque 2");
        let mut master_drive = Port::new(0x176);
        let mut control_port_base = Port::<u8>::new(0x377);
        println!("control port base : {}", control_port_base.read());
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
            if compte % 100 == 0 {
                println!("finished : {} en {}", i, compte);
            }
        }
        println!("finished : {} en {}", i, compte);
        println!("status : {}, {}, {}", _LBAlo.read(), _LBAmid.read(), _LBAhi.read());
        let mut next_port = Port::<u16>::new(0x170);
        let mut table:[u16; 512] = [0; 512];
        for i in 0..256 {
            table[i] = next_port.read();
        }

        println!("uint16_t 0 : {}", table[0]);
        println!("uint16_t 47 : {}", table[47]);
        println!("uint16_t 59 : {}", table[59]);
        println!("uint16_t 83 : {} {}", table[83], table[83] & 1024);
        println!("uint16_t 88 : {}", table[88]);
        println!("uint16_t 93 : {}", table[93]);
        println!("uint32_t 61-61 : {}", (table[60] as u32) << 0 | ((table[61] as u32) << 16));
        println!("uint32_t 100-103 : {}", ((table[100] as u64) << 0) |
                        ((table[101] as u64) << 16) |
                        ((table[102] as u64) << 32) | ((table[103] as u64) << 48));
                
                
        enable();
    }
}

pub fn read(table:[u16; 256], lba: u32, port : u16) {
    unsafe {
        disable();
        let mut master_drive = Port::new(port + 6);
   //     println!("control port base : {}", control_port_base.read());
        let mut sectorcount = Port::new(port + 2);
        let mut _LBAlo = Port::new(port + 3);
        let mut _LBAmid = Port::new(port + 4);
        let mut _LBAhi = Port::new(port + 5);
        let mut commandPort = Port::new(port + 7);
        master_drive.write(0xE0 | ((lba >> 24) & 0x0F));  // outb(0x1F6, 0xE0 | (slavebit << 4) | ((LBA >> 24) & 0x0F)) 
        sectorcount.write(0 as u8);
        _LBAlo.write(lba as u8);
        _LBAmid.write((lba >> 8) as u8);
        _LBAhi.write((lba >> 16) as u8);
    //        println!("command send");

        commandPort.write(0x20 as u8);
        let mut i = commandPort.read();
        let mut compte = 1;
        while  (i & 0x80) != 0 {
            i = commandPort.read();
            compte = 1 + compte;
            if compte % 1000000 == 0 {
                println!("not finished : {} en {}", i, compte);
            }
        }
        println!("finished : {} en {}", i, compte);
        println!("status : {}, {}, {}", _LBAlo.read(), _LBAmid.read(), _LBAhi.read());
        let mut next_port = Port::<u16>::new(port + 0);
        let mut table:[u16; 512] = [0; 512];
        for i in 0..256 {
            table[i] = next_port.read();
        }

        for i in 0..10 {
            print!("{}", ((table[i] & 0xFF) as u8) as char);
            print!("{}", (((table[i]>>8) & 0xFF) as u8) as char);
        }
        println!("");
        enable();
    }
}