use core::ptr::NonNull;

use alloc::{string::String, sync::Arc, vec::Vec};
use conquer_once::spin::OnceCell;
use spin::mutex::Mutex;

use super::{
    fs::tmpfs::{DataType, TmpFS},
    mountpoint::MountPoint,
};

pub type VFSNodeRef = Arc<Mutex<VFSNode>>;

pub static VFS_ROOT: OnceCell<VFSNodeRef> = OnceCell::uninit();

const PATH_DELIMITER: &'static str = "/";

pub struct VFSNode {
    parent: Option<VFSNodeRef>,
    children: Mutex<Vec<VFSNodeRef>>,
    name: &'static str,
    node: Arc<Node>,
}

impl VFSNode {
    pub fn has_child(&self, name: &str) -> Option<VFSNodeRef> {
        self.children
            .lock()
            .iter()
            .find(|node| node.lock().name == name)
            .cloned()
    }

    pub fn add_child(&self, name: &'static str) -> Result<VFSNodeRef, String> {
        let mut children = self.children.lock();
        let mp = match MountPoint::get_mountpoint_by_id(self.node.mountpoint_id)? {
            Some(x) => x,
            None => {
                return Err(String::from(
                    "Unable to add child to VFSNode, invalid mountpoint id!",
                ))
            }
        };
        let inode = mp.fs.lock().add_node(Vec::new());
        let result = Arc::new(Mutex::new(VFSNode {
            parent: None,
            children: Mutex::new(Vec::new()),
            name,
            node: Arc::new(Node {
                inode,
                mountpoint_id: self.node.mountpoint_id,
            }),
        }));
        children.push(result.clone());
        Ok(result)
    }

    pub fn read(&self, size: u64) -> Result<Vec<u8>, String> {
        let mp = match MountPoint::get_mountpoint_by_id(self.node.mountpoint_id)? {
            Some(x) => x,
            None => return Err(String::from("Invalid mountpoint id!")),
        };

        mp.read(self.node.inode, size)
    }

    pub fn write(&self, buffer: NonNull<u8>, size: u64) -> Result<u64, String> {
        let mp = match MountPoint::get_mountpoint_by_id(self.node.mountpoint_id)? {
            Some(x) => x,
            None => return Err(String::from("Invalid mountpoint id!")),
        };
        mp.write(self.node.inode, buffer, size)
    }
}

pub struct Node {
    inode: u64,
    mountpoint_id: u64,
}

pub fn init_vfs() {
    // We are going to mount on "/" the tmpfs file system
    let mut _tmpfs = TmpFS::new(100);
    // Given that we are using tmpfs we have to recreate the root at each reboot
    // If it was an actuall persistent file system we would have read it out of the superblock and so on
    let inode = _tmpfs.add_node(DataType::Directory(Vec::new()));
    let _mountpoint = MountPoint::new(Arc::new(Mutex::new(_tmpfs)));
    VFS_ROOT.init_once(|| {
        Arc::new(Mutex::new(VFSNode {
            parent: None,
            children: Mutex::new(Vec::new()),
            name: "",
            node: Arc::new(Node {
                inode,
                mountpoint_id: _mountpoint.id,
            }),
        }))
    });
}

fn get_vfs_node_recursive(
    path: Vec<&'static str>,
    index: usize,
    node: VFSNodeRef,
    create: bool,
) -> Result<Option<VFSNodeRef>, String> {
    if index == path.len() {
        return Ok(None);
    }
    let node_locked = node.lock();
    match node_locked.has_child(path[index]) {
        Some(x) => {
            if path.len() - 1 == index {
                return Ok(Some(x));
            }
            return get_vfs_node_recursive(path, index + 1, x, create);
        }
        None => {
            if path.len() - 1 == index && path[index] != "" && create {
                // The idea is to create the new file at this point, if the file is not in the folder and it's not an empty string
                // for instance /dev/test/
                return Ok(Some(node_locked.add_child(path[index])?));
            }
            return Ok(None);
        }
    }
}

pub fn get_vfs_node(path: &'static str, create: bool) -> Result<Option<VFSNodeRef>, String> {
    let root = match VFS_ROOT.get() {
        Some(x) => x.lock(),
        None => return Ok(None),
    };
    let tmp: Vec<&str> = path.split(PATH_DELIMITER).collect();
    if tmp.len() <= 1 {
        return Ok(None);
    }
    match root.has_child(tmp[1]) {
        Some(x) => {
            if tmp.len() == 2 {
                return Ok(Some(x));
            }
            return get_vfs_node_recursive(tmp, 2, x, create);
        }
        None => {
            return {
                if create {
                    return Ok(Some(root.add_child(tmp[1])?));
                }
                Ok(None)
            }
        }
    };
}
