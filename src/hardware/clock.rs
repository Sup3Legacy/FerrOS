use x86_64::instructions::port::Port;

use crate::print;

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

unsafe fn get_rtc(reg: u16) -> u8 {
    let mut port = Port::new(CMOS_ADDRESS);
    port.write(reg);
    let mut data = Port::new(CMOS_DATA);
    data.read()
}

unsafe fn get_update() -> u8 {
    get_rtc(0x0A) & 0x80
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
    pub unsafe fn get() -> Self {
        while get_update() != 0 {}
        let mut second = get_rtc(0x00);
        let mut minute = get_rtc(0x02);
        let mut hour = get_rtc(0x04);
        let mut day = get_rtc(0x07);
        let mut month = get_rtc(0x08);
        let mut year = get_rtc(0x09);
        let mut last_second = second;
        let mut last_minute = minute;
        let mut last_hour = hour;
        let mut last_day = day;
        let mut last_month = month;
        let mut last_year = year;

        while last_second != second
            || last_minute != minute
            || last_hour != hour
            || last_day != day
            || last_month != month
            || last_year != year
        {
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
        }
        Self {
            century: 0,
            last_second: second as usize,
            last_minute: minute as usize,
            last_hour: hour as usize,
            last_day: day as usize,
            last_month: month as usize,
            last_year: year as usize + 2000,
            last_century: 0_usize,
        }
    }
}
