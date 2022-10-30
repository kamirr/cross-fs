use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;
use std::sync::{Arc, RwLock};

use crate::filesystem::{DirectoryEntry, FileAttr, FsRead};

type NodeHandle = usize;
type FileCtx = ();

#[derive(Debug)]
pub enum Error {
    NotDirectory,
    NotFile,
    InvalidHandle,
    OutBufferError,
}

struct File(Vec<u8>);

struct Directory {
    entries: Vec<(NodeHandle, String)>,
    parent: NodeHandle,
}

impl Directory {
    fn new(parent: NodeHandle) -> Self {
        Directory {
            entries: Vec::new(),
            parent,
        }
    }
}

enum Node {
    File(File),
    Directory(Directory),
}

impl Node {
    fn as_directory(&self) -> Result<&Directory, Error> {
        match self {
            Self::File(_) => Err(Error::NotDirectory),
            Self::Directory(dir) => Ok(dir),
        }
    }

    fn as_file(&self) -> Result<&File, Error> {
        match self {
            Self::File(file) => Ok(file),
            Self::Directory(_) => Err(Error::NotFile),
        }
    }
}

pub struct MemFs {
    state: Arc<RwLock<State>>,
}

struct State {
    nodes: HashMap<NodeHandle, Node>,
    node_cnt: NodeHandle,
}

impl MemFs {
    fn next_handle(state: &mut State) -> NodeHandle {
        let handle = state.node_cnt;
        state.node_cnt += 1;

        handle
    }

    pub fn new() -> Self {
        let nodes = HashMap::new();
        let mut state = State { nodes, node_cnt: 0 };

        let root_handle = Self::next_handle(&mut state);
        let file1_handle = Self::next_handle(&mut state);
        let inner_handle = Self::next_handle(&mut state);
        let file2_handle = Self::next_handle(&mut state);

        let mut root_dir = Directory::new(root_handle);
        root_dir.entries.push((file1_handle, "file1".into()));
        root_dir.entries.push((inner_handle, "inner".into()));

        let mut inner_dir = Directory::new(root_handle);
        inner_dir.entries.push((file2_handle, "file2".into()));

        state.nodes.insert(root_handle, Node::Directory(root_dir));
        state.nodes.insert(
            file1_handle,
            Node::File(File(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])),
        );
        state.nodes.insert(inner_handle, Node::Directory(inner_dir));
        state
            .nodes
            .insert(file2_handle, Node::File(File(vec![2; 1024 * 1024])));

        MemFs {
            state: Arc::new(RwLock::new(state)),
        }
    }
}

impl FsRead for MemFs {
    type NodeHandle = NodeHandle;
    type FileCtx = FileCtx;
    type Error = Error;

    fn init(&mut self) {}
    fn destroy(&mut self) {}

    fn dir_root(&self) -> Self::NodeHandle {
        0
    }

    fn dir_read(
        &self,
        directory: &Self::NodeHandle,
        dir_data: &mut Vec<DirectoryEntry<Self>>,
    ) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        let state = self.state.read().unwrap();
        let node = state.nodes.get(directory).ok_or(Error::InvalidHandle)?;
        let dir = node.as_directory()?;

        dir_data.clear();
        dir_data.push(DirectoryEntry {
            name: "..".into(),
            handle: dir.parent,
        });
        dir_data.push(DirectoryEntry {
            name: ".".into(),
            handle: *directory,
        });
        for entry in &dir.entries {
            dir_data.push(DirectoryEntry {
                name: (&entry.1).into(),
                handle: entry.0,
            })
        }

        Ok(())
    }

    fn file_open(
        &self,
        handle: &Self::NodeHandle,
    ) -> Result<Self::FileCtx, Self::Error> {
        let state = self.state.read().unwrap();
        let node = state.nodes.get(handle).ok_or(Error::InvalidHandle)?;
        let _file = node.as_file().map_err(|_| Error::NotFile)?;

        Ok(())
    }
    
    fn file_read(
        &self,
        handle: &Self::NodeHandle,
        _ctx: &Self::FileCtx,
        range: Range<usize>,
        mut buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        let state = self.state.read().unwrap();
        let node = state.nodes.get(handle).ok_or(Error::InvalidHandle)?;
        let file = node.as_file()?;

        buf.write_all(&file.0[range.clone()])
            .map_err(|_| Error::OutBufferError)?;

        Ok(range.end - range.start)
    }

    fn file_get_attr(&self, _file: &Self::NodeHandle) -> Result<FileAttr, Self::Error> {
        Ok(FileAttr)
    }

    fn file_close(&self, file: &Self::NodeHandle, _ctx: Self::FileCtx) -> Result<(), Self::Error> {
        let state = self.state.read().unwrap();
        let node = state.nodes.get(file).ok_or(Error::InvalidHandle)?;
        let _file = node.as_file()?;

        Ok(())
    }
}
