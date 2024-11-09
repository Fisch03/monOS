use alloc::{
    boxed::Box,
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::any::Any;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{Path, Read, Seek, Write};

use spin::{RwLock, RwLockReadGuard};

pub struct VFS {
    root: Arc<VFSNode>,
}

impl VFS {
    pub fn new() -> Self {
        VFS {
            root: VFSNode::new(String::from("/"), VFSNodeType::Directory),
        }
    }

    pub fn get<'p, P: Into<Path<'p>>>(&self, path: P) -> Option<Arc<VFSNode>> {
        self.root.clone().get(path)
    }
}

impl core::ops::Deref for VFS {
    type Target = VFSNode;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

pub struct VFSNode {
    name: String,
    node_type: VFSNodeType,

    parent: Option<Weak<VFSNode>>,
    children: RwLock<Vec<Arc<VFSNode>>>,

    fs: RwLock<Option<FSData>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VFSNodeType {
    Directory,
    File { size: usize },
}

pub struct FSData {
    pub(super) fs: Arc<dyn FileSystem>,
    pub(super) data: Box<dyn Any + Send + Sync>,
}

impl FSData {
    pub fn new<FS: FileSystem + 'static, D: Send + Sync + 'static>(fs: FS, data: D) -> Self {
        FSData {
            fs: Arc::new(fs),
            data: Box::new(data),
        }
    }

    pub fn data<T: 'static>(&self) -> &T {
        self.data.downcast_ref().unwrap()
    }

    // pub(super) fn data_mut<T: 'static>(&mut self) -> &mut T {
    //     self.data.downcast_mut().unwrap()
    // }
}

impl core::fmt::Debug for FSData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FSData")
            .field("fs", &self.fs)
            .field("data", &"Box<dyn Any>")
            .finish()
    }
}

impl VFSNode {
    pub(super) fn new(name: String, node_type: VFSNodeType) -> Arc<VFSNode> {
        Arc::new(VFSNode {
            name,
            node_type,
            parent: None,
            children: RwLock::new(Vec::new()),
            fs: RwLock::new(None),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_directory(&self) -> bool {
        matches!(self.node_type, VFSNodeType::Directory)
    }
    pub fn is_file(&self) -> bool {
        matches!(self.node_type, VFSNodeType::File { .. })
    }

    pub fn parent(&self) -> Option<Arc<VFSNode>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }
    pub fn children(self: &Arc<VFSNode>) -> RwLockReadGuard<Vec<Arc<VFSNode>>> {
        self.list();
        self.children.read()
    }
    pub(super) fn add_child(
        parent: &Arc<VFSNode>,
        name: String,
        node_type: VFSNodeType,
        fs: Option<FSData>,
    ) {
        let child = Arc::new(VFSNode {
            name,
            node_type,
            parent: Some(Arc::downgrade(parent)),
            children: RwLock::new(Vec::new()),
            fs: RwLock::new(fs),
        });

        parent.children.write().push(child);
    }

    fn list(self: &Arc<VFSNode>) {
        let fs = self.fs.read();
        if let Some(fs) = fs.as_ref() {
            self.children.write().clear();
            fs.fs.clone().list(self.clone());
        }
    }

    pub fn get<'p, P: Into<Path<'p>>>(self: &Arc<VFSNode>, path: P) -> Option<Arc<VFSNode>> {
        let path = path.into();

        // TODO: only do a list if the node wasnt found otherwise
        self.list();

        let children = self.children.read();
        let children = children.as_slice();

        if let Some((current_dir, path_children)) = path.enter() {
            for node in children {
                if node.name == current_dir.as_str() {
                    return node.get(path_children);
                }
            }

            None
        } else {
            for node in children {
                if node.name() == path.as_str() {
                    return Some(node.clone());
                }
            }
            None
        }
    }

    pub fn fs(&self) -> RwLockReadGuard<Option<FSData>> {
        self.fs.read()
    }

    pub(super) fn set_fs(&self, fs: FSData) {
        *self.fs.write() = Some(fs);
    }

    pub fn open(&self) -> Result<File, OpenError> {
        if !self.is_file() {
            return Err(OpenError::NotAFile);
        }

        let fs = self.fs.read();
        if let Some(fs) = fs.as_ref() {
            let fs = fs.fs.clone();
            fs.open(self)
        } else {
            Err(OpenError::NotFound)
        }
    }

    pub fn mount<FS: FileSystem + 'static>(&self, fs: FS) -> Result<(), MountError> {
        if self.node_type != VFSNodeType::Directory {
            return Err(MountError::NotADirectory);
        }

        if !self.children.read().is_empty() {
            return Err(MountError::NotEmpty);
        }

        *self.fs.write() = None;
        fs.mount(self);
        debug_assert!(self.fs.read().is_some());

        Ok(())
    }
}

impl core::fmt::Debug for VFSNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VFSNode")
            .field("name", &self.name)
            .field("node_type", &self.node_type)
            .field("children", &self.children)
            .finish()
    }
}

#[derive(Debug)]
pub enum MountError {
    NotADirectory,
    NotEmpty,
}

pub trait FileSystem: Send + Sync + core::fmt::Debug {
    fn open(self: Arc<Self>, node: &VFSNode) -> Result<File, OpenError>;
    fn close(&self, file: File) -> Result<(), CloseError>;
    fn list(self: Arc<Self>, node: Arc<VFSNode>);
    // fn create(self: Arc<Self>, parent: &VFSNode, name: &str) -> Result<VFSNode, CreateError>;
    // fn mkdir(self: Arc<Self>, parent: &VFSNode, name: &str) -> Result<VFSNode, CreateError>;
    // fn rename(&self, node: &VFSNode, new_name: &str) -> Result<(), RenameError>;
    // fn remove(&self, node: &VFSNode) -> Result<(), RemoveError>;
    //
    fn read(&self, file: &File, buf: &mut [u8]) -> usize;
    fn write(&self, file: &mut File, buf: &[u8]) -> usize;
    fn seek(&self, file: &File, pos: usize);

    fn mount(self, node: &VFSNode);
}

#[derive(Debug)]
pub enum OpenError {
    NotFound,
    NotAFile,
}

#[derive(Debug)]
pub enum CloseError {
    NotOpen,
    InUse,
}

#[derive(Debug)]
pub enum CreateError {
    ReadOnly,
    AlreadyExists,
    NotADirectory,
}

#[derive(Debug)]
pub enum RenameError {
    ReadOnly,
    AlreadyExists,
    InUse,
}

#[derive(Debug)]
pub enum RemoveError {
    ReadOnly,
    InUse,
}

#[derive(Debug)]
pub struct File {
    pub(super) name: String,
    pub(super) size: usize,
    pub(super) pos: AtomicUsize,

    fs: FSData,
}

impl File {
    pub(super) fn new<T: Send + Sync + 'static>(
        name: String,
        size: usize,
        fs: Arc<dyn FileSystem>,
        fs_data: T,
    ) -> Self {
        File {
            name,
            size,
            pos: AtomicUsize::new(0),
            fs: FSData {
                fs,
                data: Box::new(fs_data),
            },
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn data<T: 'static>(&self) -> &T {
        self.fs.data.downcast_ref().unwrap()
    }

    pub(super) fn data_mut<T: 'static>(&mut self) -> &mut T {
        self.fs.data.downcast_mut().unwrap()
    }

    pub fn close(self) -> Result<(), CloseError> {
        let fs = self.fs.fs.clone();
        fs.close(self)
    }
}

impl Read for File {
    fn read(&self, buf: &mut [u8]) -> usize {
        self.fs.fs.read(self, buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> usize {
        let fs = self.fs.fs.clone(); // this clonage is a bit annoying but oh well
        fs.write(self, buf)
    }
}

impl Seek for File {
    fn set_pos(&self, pos: usize) {
        self.pos.store(pos, Ordering::Relaxed);
        self.fs.fs.seek(self, pos);
    }

    fn get_pos(&self) -> usize {
        self.pos.load(Ordering::Relaxed)
    }
}
