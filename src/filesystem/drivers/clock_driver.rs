use super::super::partition::Partition;
use crate::{data_storage::path::Path, warningln};

use alloc::format;
use alloc::vec::Vec;
use x86_64::instructions::port::Port;

pub struct ClockDriver;

const CMOS_ADDRESS: u16 = 0x70;
static mut CMOS_ADDRESS_PORT: Port<u8> = Port::new(CMOS_ADDRESS);
const CMOS_DATA: u16 = 0x71;
static mut CMOS_DATA_PORT: Port<u8> = Port::new(CMOS_DATA);

/// # Safety
/// TODO
unsafe fn get_rtc(reg: u8) -> u8 {
    CMOS_ADDRESS_PORT.write(reg);
    CMOS_DATA_PORT.read()
}

/// # Safety
/// TODO
unsafe fn wait_update() {
    //while get_rtc(0x0A) & 0x80 != 0x80 {}
    while get_rtc(0x0A) & 0x80 == 0x80 {}
}

fn cvt_bcd(value: usize) -> usize {
    (value & 0xF) + ((value / 16) * 10)
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

    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }
}

#[derive(Debug)]
pub struct Time {
    second: usize,
    minute: usize,
    hour: usize,
    workday: usize,
    day: usize,
    month: usize,
    year: usize,
    century: usize,
}

impl Time {
    /// # Safety
    /// TODO
    pub unsafe fn get() -> Self {
        asm!("cli");
        let mut second;
        let mut minute;
        let mut hour;
        let workday;
        let mut day;
        let mut month;
        let mut year;
        let century;
        let register_b;
        wait_update();
        second = get_rtc(0x00) as usize;
        minute = get_rtc(0x02) as usize;
        hour = get_rtc(0x04) as usize;
        workday = get_rtc(0x06) as usize;
        day = get_rtc(0x07) as usize;
        month = get_rtc(0x08) as usize;
        year = get_rtc(0x09) as usize;
        century = 21_usize;
        register_b = get_rtc(0x0b);

        if register_b & 4 != 4 {
            second = cvt_bcd(second);
            minute = cvt_bcd(minute);
            hour = cvt_bcd(hour & 0x7F) | (hour & 0x80);
            day = cvt_bcd(day);
            month = cvt_bcd(month);
            year = cvt_bcd(year);
        }

        if register_b & 2 != 2 || hour & 0x80 == 0x80 {
            hour = ((hour & 0x7F) + 12) % 24;
        }

        asm!("sti");
        Self {
            century,
            second,
            minute,
            hour,
            workday,
            day,
            month,
            year: year + 100 * (century - 1),
        }
    }
}

impl ClockDriver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClockDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl Partition for ClockDriver {
    fn read(&self, _path: &Path, _offset: usize, _size: usize) -> Vec<u8> {
        let time = unsafe { Time::get() };
        let string = format!(
            "{} {} {} {} {} {} {} {}",
            time.century,
            time.year,
            time.month,
            time.day,
            WeekDay::from_int(time.workday as u8).to_string(),
            time.hour,
            time.minute,
            time.second,
        );
        let vec = Vec::from(string.as_bytes());
        vec
    }

    fn write(&mut self, _path: &Path, _buffer: &[u8], _offset: usize, _flags: u64) -> isize {
        warningln!("User-program attempted to write in clock.");
        -1
    }

    fn lseek(&self) {
        todo!()
    }

    fn flush(&self) {
        todo!()
    }

    fn read_raw(&self) {
        todo!()
    }
}
