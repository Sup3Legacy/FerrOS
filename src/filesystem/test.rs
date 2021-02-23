use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts::{disable, enable};
use crate::{print, println};

pub fn test() {
    unsafe {
        //println!("data read {}", get_total_head());
        let mut master_drive = Port::new(0x1F6);

        let mut sectorcount = Port::new(0x1F2);
        let mut _LBAlo = Port::new(0x1F3);
        let mut _LBAmid = Port::new(0x1F4);
        let mut _LBAhi = Port::new(0x1F5);
        let mut commandPort = Port::new(0x1F7);
        disable();
        master_drive.write(0xA0 as u8);
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


        enable();
    }
}


#[naked]
unsafe extern "C" fn get_total_head() -> u64 {
    asm!(
        "mov ah, 0x41",
        "mov bx, 0x55AA",
        "mov dl, 0x80",
        "int 0x13",
        "mov ah, dh",
        "mov al, cl",
        "ret"
    );
    loop {}
}