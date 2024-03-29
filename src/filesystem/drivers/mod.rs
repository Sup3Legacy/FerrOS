//! Drivers that can be attached to the `VFS`

pub mod clock_driver;
pub mod disk_operations;
pub mod fifo;
pub mod hardware;
pub mod host_shell;
pub mod keyboard;
pub mod mouse_driver;
pub mod nopart;
pub mod proc;
pub mod ramdisk;
pub mod screen_partition;
pub mod software;
pub mod sound;
pub mod ustar;
