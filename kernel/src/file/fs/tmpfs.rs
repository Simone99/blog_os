use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{string::String, sync::Arc, vec::Vec};
use spin::Mutex;

use super::Filesystem;

static INODE_ID: AtomicU64 = AtomicU64::new(0);

pub enum DataType {
    Regular(Vec<u8>),
    Directory(Vec<DirType>),
    Link(Vec<u8>),
}

pub struct DirType {
    name: String,
    inode: u64,
    entry_type: DataType,
}

impl DirType {
    pub fn new(name: String, inode: u64, entry_type: DataType) -> Self {
        Self {
            name,
            inode,
            entry_type,
        }
    }
}

struct Node {
    data_type: DataType,
}

impl Node {
    pub fn new(data_type: DataType) -> Self {
        Self { data_type }
    }
}

pub struct TmpFS {
    max_size: u64,
    size: usize,
    readonly: bool,
    nodes: Mutex<Vec<Arc<Mutex<Node>>>>,
}

impl TmpFS {
    pub fn new(max_size: u64) -> Self {
        Self {
            max_size,
            size: 0,
            readonly: false,
            nodes: Mutex::new(Vec::new()),
        }
    }

    pub fn add_node(&mut self, data_type: DataType) -> u64 {
        let node = Node::new(data_type);
        self.nodes.lock().push(Arc::new(Mutex::new(node)));
        INODE_ID.fetch_add(1, Ordering::AcqRel)
    }
}

impl Filesystem for TmpFS {
    fn read(&self, inode: u64, size: u64) -> Result<Vec<u8>, String> {
        let binding = self.nodes.lock();
        let node = match binding.get(inode as usize) {
            Some(x) => x,
            None => return Err(String::from("The specified iNode does not exist!")),
        };
        let node_locked = node.lock();
        match &node_locked.data_type {
            DataType::Regular(vec) => {
                if (size as usize) < vec.len() {
                    let mut tmp_vec = vec.clone();
                    let _ = tmp_vec.split_off(size as usize);
                    return Ok(tmp_vec.clone());
                }
                Ok(vec.clone())
            }
            DataType::Directory(_vec) => Err(String::from("Trying to read data from a directory!")),
            DataType::Link(_vec) => Err(String::from("Trying to read data from a link!")),
        }
    }

    fn add_node(&mut self, data: Vec<u8>) -> u64 {
        self.add_node(DataType::Regular(data))
    }

    fn write(&self, inode: u64, buffer: NonNull<u8>, size: u64) -> Result<u64, String> {
        let binding = self.nodes.lock();
        let node = match binding.get(inode as usize) {
            Some(x) => x,
            None => return Err(String::from("The specified iNode does not exist!")),
        };
        let mut node_locked = node.lock();
        node_locked.data_type = DataType::Regular(unsafe {
            Vec::from_raw_parts(buffer.as_ptr(), size as usize, size as usize)
        });
        Ok(size)
    }
}
