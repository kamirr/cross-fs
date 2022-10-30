use std::{
    ffi::OsString,
    fmt::Debug,
    ops::Range,
};

pub struct DirectoryEntry<FS: FsRead> {
    pub name: OsString,
    pub handle: FS::NodeHandle,
}

pub struct FileAttr;

pub trait FsRead {
    type FileCtx;
    type NodeHandle: Clone;
    type Error: Debug;

    fn init(&mut self);
    fn destroy(&mut self);

    fn dir_root(&self) -> Self::NodeHandle;
    fn dir_read(
        &self,
        directory: &Self::NodeHandle,
        dir_data: &mut Vec<DirectoryEntry<Self>>,
    ) -> Result<(), Self::Error>
    where
        Self: Sized;

    fn file_open(
        &self,
        handle: &Self::NodeHandle,
    ) -> Result<Self::FileCtx, Self::Error>;
    fn file_read(
        &self,
        handle: &Self::NodeHandle,
        ctx: &Self::FileCtx,
        range: Range<usize>,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error>;
    fn file_get_attr(&self, file: &Self::NodeHandle) -> Result<FileAttr, Self::Error>;
    fn file_close(&self, file: &Self::NodeHandle, ctx: Self::FileCtx) -> Result<(), Self::Error>;
}
