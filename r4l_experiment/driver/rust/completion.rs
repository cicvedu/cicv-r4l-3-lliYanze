// SPDX-License-Identifier: GPL-2.0

//! Rust character device sample.

use core::result::Result::Err;

use kernel::prelude::*;
use kernel::sync::CondVar;
use kernel::sync::Mutex;
use kernel::{chrdev, file};
const GLOBALMEM_SIZE: usize = 0x1000;

module! {
    type: Completion,
    name: "rust_chrdev",
    author: "Rust for Linux Contributors",
    description: "Rust character device sample",
    license: "GPL",
}

static GLOBALMEM_BUF: Mutex<[u8; GLOBALMEM_SIZE]> = unsafe { Mutex::new([0u8; GLOBALMEM_SIZE]) };

kernel::init_static_sync! {
    static GLOBALMUTEX: Mutex<bool> = false;
    static GLOBALCOND: CondVar ;
}

struct RustFile {
    #[allow(dead_code)]
    inner: &'static Mutex<[u8; GLOBALMEM_SIZE]>,
}

#[vtable]
impl file::Operations for RustFile {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        pr_info!("open device\n");
        Ok(Box::try_new(RustFile {
            inner: &GLOBALMEM_BUF,
        })?)
    }

    fn write(
        _this: &Self,
        _file: &file::File,
        _reader: &mut impl kernel::io_buffer::IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        // pr_info!("write to device\n");
        let offset = _offset.try_into()?;
        let mut buffer = _this.inner.lock();
        let len = core::cmp::min(_reader.len(), buffer.len() - offset as usize);
        _reader.read_slice(&mut buffer[offset..][..len])?;

        // 通知写入完成
        GLOBALCOND.notify_all();
        core::result::Result::Ok(len)
    }

    fn read(
        _this: &Self,
        _file: &file::File,
        _writer: &mut impl kernel::io_buffer::IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        // 等待写入完成
        pr_info!("2wait read for device\n");
        let mut write_completed = GLOBALMUTEX.lock();
        pr_info!("*\n");
        let _ = GLOBALCOND.wait(&mut write_completed);

        let offset = _offset.try_into()?;
        let buffer = _this.inner.lock();
        let len = core::cmp::min(_writer.len(), buffer.len() - offset as usize);
        _writer.write_slice(&buffer[offset..][..len])?;

        core::result::Result::Ok(len)
    }
}

struct Completion {
    _dev: Pin<Box<chrdev::Registration<2>>>,
}

impl kernel::Module for Completion {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust character device sample (init)\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        chrdev_reg.as_mut().register::<RustFile>()?;
        // chrdev_reg.as_mut().register::<RustFile>()?;

        Ok(Completion { _dev: chrdev_reg })
    }
}

impl Drop for Completion {
    fn drop(&mut self) {
        pr_info!("Rust character device sample (exit)\n");
    }
}
