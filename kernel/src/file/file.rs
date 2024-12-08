use core::ptr::NonNull;

use alloc::{string::String, sync::Arc, vec::Vec};
use conquer_once::spin::OnceCell;
use spin::Mutex;

use super::vfs::{get_vfs_node, VFSNodeRef};

pub struct File {
    vfs_node: VFSNodeRef,
}

impl File {
    pub fn open(path: &'static str) -> Result<u64, String> {
        // First I need to find the corresponding VFS node
        let vfs_node = match get_vfs_node(path, true)? {
            Some(node) => node,
            None => {
                // In this case we should create the file
                return Err(String::from("Unable to create file, check your path!"));
            }
        };
        let result = Self { vfs_node };
        match OPEN_FILE_TABLE.get() {
            Some(oft) => {
                let mut oft_locked = oft.lock();
                oft_locked.push(OFTEntry {
                    file: Arc::new(result),
                });
                return Ok((oft_locked.len() - 1) as u64);
            }
            None => return Err(String::from("Open File Table not initialized!")),
        }
    }

    pub fn read(fd: u64, buffer: NonNull<u8>, count: u64) -> Result<u64, String> {
        let oft = match OPEN_FILE_TABLE.get() {
            Some(x) => x.lock(),
            None => return Err(String::from("Open File Table not initialized!")),
        };
        let file = match oft.get(fd as usize) {
            Some(f) => f.file.clone(),
            None => return Err(String::from("Invalid file descriptor!")),
        };
        let mut data: Vec<u8> = file.vfs_node.lock().read(count)?;
        if data.len() > 0 {
            let data_ptr = match NonNull::new(data.as_mut_slice().as_mut_ptr()) {
                Some(x) => x,
                None => {
                    return Err(String::from(
                        "Reading data from file returned a null pointer!",
                    ))
                }
            };
            unsafe { buffer.copy_from(data_ptr, count as usize) };
        }
        Ok(data.len() as u64)
    }

    pub fn write(fd: u64, buffer: NonNull<u8>, count: u64) -> Result<u64, String> {
        let oft = match OPEN_FILE_TABLE.get() {
            Some(x) => x.lock(),
            None => return Err(String::from("Open File Table not initialized!")),
        };
        let file = match oft.get(fd as usize) {
            Some(f) => f.file.clone(),
            None => return Err(String::from("Invalid file descriptor!")),
        };
        if count > 0 {
            return file.vfs_node.lock().write(buffer, count);
        }
        Ok(0)
    }
}

pub struct OFTEntry {
    file: Arc<File>,
}

type OFT = Arc<Mutex<Vec<OFTEntry>>>;

static OPEN_FILE_TABLE: OnceCell<OFT> = OnceCell::uninit();

pub fn init_oft() {
    OPEN_FILE_TABLE.init_once(|| Arc::new(Mutex::new(Vec::new())));
}
