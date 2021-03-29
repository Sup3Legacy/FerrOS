use x86_64::instructions::port::Port;

use crate::println;

const CMOS_ADDRESS: u16 = 0x70;
static mut CMOS_ADDRESS_PORT: Port<u16> = Port::new(CMOS_ADDRESS);
const CMOS_DATA: u16 = 0x71;
static mut CMOS_DATA_PORT: Port<u8> = Port::new(CMOS_DATA);

/// # Safety
/// TODO
unsafe fn get_rtc(reg: u16) -> u8 {
    CMOS_ADDRESS_PORT.write(reg);
    CMOS_DATA_PORT.read()
}

/// # Safety
/// TODO
unsafe fn get_update() -> u8 {
    get_rtc(0x0A) & 0x80
}

#[derive(Debug)]
pub enum WeekDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl WeekDay {
    pub fn from_int(index: u8) -> Self {
        match index {
            2 => Self::Monday,
            3 => Self::Tuesday,
            4 => Self::Wednesday,
            5 => Self::Thursday,
            6 => Self::Friday,
            7 => Self::Saturday,
            1 => Self::Sunday,
            _ => Self::Monday, // TO DO : ERROR
        }
    }
}

#[derive(Debug)]
pub struct Time {
    century: usize,
    last_second: usize,
    last_minute: usize,
    last_hour: usize,
    last_day: usize,
    last_month: usize,
    last_year: usize,
    last_century: usize,
}

impl Time {
    /// # Safety
    /// TODO
    pub unsafe fn get() -> Self {
        while get_update() != 0 {}
        let mut second = get_rtc(0x00);
        let mut minute = get_rtc(0x02);
        let mut hour = get_rtc(0x04);
        let mut day = get_rtc(0x07);
        let mut month = get_rtc(0x08);
        let mut year = get_rtc(0x09);
        // + 2 to enter the while
        let mut last_second;
        let mut last_minute;
        let mut last_hour;
        let mut last_day;
        let mut last_month;
        let mut last_year;

        loop {
            println!("Clock update.");
            last_second = second;
            last_minute = minute;
            last_hour = hour;
            last_day = day;
            last_month = month;
            last_year = year;
            while get_update() != 0 {}
            second = get_rtc(0x00);
            minute = get_rtc(0x02);
            hour = get_rtc(0x04);
            day = get_rtc(0x07);
            month = get_rtc(0x08);
            year = get_rtc(0x09);
            if !(last_second != second
                || last_minute != minute
                || last_hour != hour
                || last_day != day
                || last_month != month
                || last_year != year)
            {
                break;
            }
        }
        Self {
            century: 0,
            last_second: ((second & 0x0F) + ((second >> 4) * 10)) as usize,
            last_minute: ((minute & 0x0F) + ((minute >> 4) * 10)) as usize,
            last_hour: ((hour & 0x0F) + ((hour >> 4) * 10)) as usize,
            last_day: ((day & 0x0F) + ((day >> 4) * 10)) as usize,
            last_month: ((month & 0x0F) + ((month >> 4) * 10)) as usize,
            last_year: ((year & 0x0F) + ((year >> 4) * 10)) as usize + 2000,
            last_century: 0_usize,
        }
    }
}
