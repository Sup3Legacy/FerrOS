use bit_field::BitField;
use x86_64::instructions::port::Port;

enum MouseBytes {
    CommandByte = 0xD4,
    ACK = 0xFA,
    Reset = 0xFF,
    Resend = 0xFE,
    DisablePacketStreaming = 0xF5,
    EnablePacketStreaming = 0xF4,
    SetSampleRate = 0xF3,
    GetID = 0xF2,
    RequestPacket = 0xEB,
    StatusRequest = 0xE9,
    SetResolution = 0xE8,
}

impl MouseBytes {
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseError {
    MouseNotPresent,
    UnknownError,
    FailedIRQInit,
}

#[derive(Debug, Copy, Clone)]
pub struct MouseInfo {
    y_overflow: bool,
    x_overflow: bool,
    y_sign: bool,
    x_sign: bool,
    always_on: bool,
    middle_button: bool,
    right_button: bool,
    left_button: bool,
    x_movement: u8,
    y_movement: u8,
}

impl MouseInfo {
    pub fn new(
        y_overflow: bool,
        x_overflow: bool,
        y_sign: bool,
        x_sign: bool,
        always_on: bool,
        middle_button: bool,
        right_button: bool,
        left_button: bool,
        x_movement: u8,
        y_movement: u8,
    ) -> Self {
        Self {
            y_overflow,
            x_overflow,
            y_sign,
            x_sign,
            always_on,
            middle_button,
            right_button,
            left_button,
            x_movement,
            y_movement,
        }
    }
}

fn poll_controller_write() {
    let mut controller_port: Port<u8> = Port::new(0x64);
    while unsafe { controller_port.read() } & 2 != 0 {}
}

fn poll_controller_read() {
    let mut controller_port: Port<u8> = Port::new(0x64);
    while unsafe { controller_port.read() } & 1 == 0 {}
}

fn send_to_mouse(data: u8) {
    let mut keyboard_port: Port<u8> = Port::new(0x60);
    let mut controller_port: Port<u8> = Port::new(0x64);
    poll_controller_write();
    unsafe { controller_port.write(MouseBytes::CommandByte.to_byte()) };
    poll_controller_write();
    unsafe { keyboard_port.write(data) };
}

fn send_to_controller(data: u8) {
    let mut controller_port: Port<u8> = Port::new(0x64);
    poll_controller_write();
    unsafe { controller_port.write(data) };
}

fn read_mouse_byte() -> u8 {
    let mut keyboard_port: Port<u8> = Port::new(0x60);
    poll_controller_read();
    unsafe { keyboard_port.read() }
}

fn read_controller() -> u8 {
    let mut controller_port: Port<u8> = Port::new(0x64);
    poll_controller_read();
    unsafe { controller_port.read() }
}

pub fn read_simple_packet() -> MouseInfo {
    let misc = read_mouse_byte();
    let x = read_mouse_byte();
    let y = read_mouse_byte();
    MouseInfo::new(
        misc.get_bit(7),
        misc.get_bit(6),
        misc.get_bit(5),
        misc.get_bit(4),
        misc.get_bit(3),
        misc.get_bit(2),
        misc.get_bit(1),
        misc.get_bit(0),
        x,
        y,
    )
}

fn enable_irq() -> Result<(), MouseError> {
    let mut keyboard_port: Port<u8> = Port::new(0x60);
    let mut controller_port: Port<u8> = Port::new(0x64);
    send_to_controller(0x20);
    let mut status_byte = read_controller();
    status_byte |= 2;
    status_byte &= !0x20;
    send_to_controller(0x60);
    send_to_controller(status_byte);
    if read_mouse_byte() == MouseBytes::ACK.to_byte() {
        Ok(())
    } else {
        Err(MouseError::FailedIRQInit)
    }
}

pub fn init() -> Result<(), MouseError> {
    enable_irq()
}
