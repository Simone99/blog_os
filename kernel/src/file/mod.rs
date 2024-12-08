use core::ptr::NonNull;

use alloc::string::String;
use file::{init_oft, File};
use vfs::init_vfs;

use crate::println;

pub mod file;
pub mod fs;
pub mod mountpoint;
pub mod vfs;

pub fn init() -> Result<(), String> {
    // We need first to mount the root filesystem to "/"
    // In our case we will always use the tmpfs

    // Create first the mount point on "/" creating the filesystem as well
    // Create the VFS node
    // Create all filesystem nodes
    init_vfs();

    // Initialize the global open file table
    init_oft();

    // Let's try creating a file
    let fd = File::open("/test.txt")?;

    // Let's try reading data from the file
    let mut buffer = [0u8; 4];
    let buffer_ptr = match NonNull::new(buffer.as_mut_ptr()) {
        Some(x) => x,
        None => {
            return Err(String::from(
                "Buffer initialization for read, returned a null pointer!",
            ))
        }
    };
    let size_read = File::read(fd, buffer_ptr, buffer.len() as u64)?;
    println!("Before write:\nSize: {} Data: {:#?}", size_read, buffer);

    // Let's try writing to a file
    let mut buffer_write = [0u8, 1, 2, 3];
    let buffer_ptr_write = match NonNull::new(buffer_write.as_mut_ptr()) {
        Some(x) => x,
        None => {
            return Err(String::from(
                "Buffer initialization for write, returned a null pointer!",
            ))
        }
    };
    let size_write = File::write(fd, buffer_ptr_write, buffer_write.len() as u64)?;
    println!("Wrote {} bytes", size_write);

    // Let's see if the changes took place with a read
    buffer = [0u8; 4];
    let buffer_ptr = match NonNull::new(buffer.as_mut_ptr()) {
        Some(x) => x,
        None => {
            return Err(String::from(
                "Buffer initialization for second read, returned a null pointer!",
            ))
        }
    };
    let size_read = File::read(fd, buffer_ptr, buffer.len() as u64)?;
    println!("After write:\nSize: {} Data: {:#?}", size_read, buffer);

    Ok(())
}
