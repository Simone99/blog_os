use core::{
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::{string::String, sync::Arc, vec::Vec};
use conquer_once::spin::OnceCell;
use spin::Mutex;

use super::fs::Filesystem;

static MOUNTPOINTS: OnceCell<Arc<Mutex<Vec<Arc<MountPoint>>>>> = OnceCell::uninit();

static MOUNTPOINT_ID: AtomicU64 = AtomicU64::new(0);
pub struct MountPoint {
    pub id: u64,
    pub fs: Arc<Mutex<dyn Filesystem>>,
}

impl MountPoint {
    // Creating a new mountpoint will automatically register it into the list of mountpoints
    pub fn new(fs: Arc<Mutex<dyn Filesystem>>) -> Arc<Self> {
        let result = Arc::new(Self {
            id: MOUNTPOINT_ID.fetch_add(1, Ordering::AcqRel),
            fs,
        });
        let mount_points = MOUNTPOINTS.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
        mount_points.lock().push(result.clone());
        result
    }

    // I think calling this function before initializing the list of mountpoints should lead to an error
    pub fn get_mountpoint_by_id(id: u64) -> Result<Option<Arc<Self>>, String> {
        let mount_points = match MOUNTPOINTS.get() {
            Some(x) => x.lock(),
            None => {
                return Err(String::from(
                    "get_mountpoint_by_id has been called before MOUNTPOINTS got initialized!",
                ));
            }
        };
        for mount_point in mount_points.iter() {
            if mount_point.id == id {
                return Ok(Some(mount_point.clone()));
            }
        }
        return Ok(None);
    }

    pub fn read(&self, inode: u64, size: u64) -> Result<Vec<u8>, String> {
        let tmp = self.fs.lock();
        tmp.read(inode, size)
    }
    pub fn write(&self, inode: u64, buffer: NonNull<u8>, size: u64) -> Result<u64, String> {
        let tmp = self.fs.lock();
        tmp.write(inode, buffer, size)
    }
}
