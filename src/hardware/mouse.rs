use bit_field::BitField;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::port::Port;

/// Queue of mouse packets
static MOUSE_QUEUE: OnceCell<ArrayQueue<MousePacket>> = OnceCell::uninit();

/// Max size of the queue of mouse packets
const MOUSE_QUEUE_CAP: usize = 256;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code, clippy::upper_case_acronyms)]
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
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseError {
    MouseNotPresent,
    UnknownError,
    FailedIRQInit,
    QueueNotPresent,
    QueueFull,
}

#[derive(Debug, Copy, Clone)]
pub struct MousePacket {
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

impl MousePacket {
    #[allow(clippy::too_many_arguments)]
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
    pub fn to_bytes(&self) -> (u8, u8, u8) {
        let mut first = 0_u8;
        first.set_bit(0, self.left_button);
        first.set_bit(1, self.middle_button);
        first.set_bit(2, self.right_button);
        first.set_bit(3, self.always_on);
        first.set_bit(4, self.x_sign);
        first.set_bit(5, self.y_sign);
        first.set_bit(6, self.x_overflow);
        first.set_bit(7, self.y_overflow);
        (first, self.x_movement, self.y_movement)
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

#[allow(dead_code)]
fn read_controller() -> u8 {
    let mut controller_port: Port<u8> = Port::new(0x64);
    unsafe { controller_port.read() }
}

pub fn read_simple_packet() {
    let misc = read_mouse_byte();
    let x = read_mouse_byte();
    let y = read_mouse_byte();
    let packet = MousePacket::new(
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
    );
    enqueue_packet(packet).unwrap();
}

fn enable_irq() -> Result<(), MouseError> {
    let mut keyboard_port: Port<u8> = Port::new(0x60);
    let mut controller_port: Port<u8> = Port::new(0x64);
    //mouse_wait(1);
    //outportb(0x64, 0xA8);
    send_to_controller(0xA8);
    send_to_controller(0x20);

    poll_controller_read();
    let mut status_byte = (unsafe { keyboard_port.read() } | 2);
    status_byte &= !0x20;

    poll_controller_write();

    unsafe { controller_port.write(0x60) };
    poll_controller_write();
    unsafe { keyboard_port.write(status_byte) };

    send_to_mouse(0xF6);
    read_mouse_byte();
    send_to_mouse(MouseBytes::EnablePacketStreaming.to_byte());
    read_mouse_byte();
    Ok(())
}

pub fn init() -> Result<(), MouseError> {
    enable_irq()
}

fn enqueue_packet(packet: MousePacket) -> Result<(), MouseError> {
    if let Ok(queue) = MOUSE_QUEUE.try_get() {
        if queue.len() >= MOUSE_QUEUE_CAP {
            return Err(MouseError::QueueFull);
        }
        match queue.push(packet) {
            Ok(()) => Ok(()),
            _ => Err(MouseError::UnknownError),
        }
    } else {
        Err(MouseError::QueueNotPresent)
    }
}

pub fn get_packet() -> Option<MousePacket> {
    if let Ok(queue) = MOUSE_QUEUE.try_get() {
        match queue.pop() {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    } else {
        None
    }
}
