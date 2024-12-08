use core::ptr::NonNull;

use alloc::{string::String, sync::Arc, vec::Vec};
use conquer_once::spin::OnceCell;
use spin::Mutex;

pub mod tmpfs;

pub static FILESYSTEMS: OnceCell<Arc<Mutex<Vec<Arc<dyn Filesystem>>>>> = OnceCell::uninit();

pub trait Filesystem: Send + Sync + 'static {
    fn read(&self, inode: u64, size: u64) -> Result<Vec<u8>, String>;
    fn write(&self, inode: u64, buffer: NonNull<u8>, size: u64) -> Result<u64, String>;
    fn add_node(&mut self, data: Vec<u8>) -> u64;
}
