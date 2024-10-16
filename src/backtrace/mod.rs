// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use crate::Symbol;
use libc::c_void;
use std::path::PathBuf;

pub trait AsSymbol: Sized {
    fn name(&self) -> Option<Vec<u8>>;
    fn addr(&self) -> Option<*mut c_void>;
    fn lineno(&self) -> Option<u32>;
    fn filename(&self) -> Option<PathBuf>;
}

impl AsSymbol for backtrace::Symbol {
    fn name(&self) -> Option<Vec<u8>> {
        self.name().map(|name| name.as_bytes().to_vec())
    }

    fn addr(&self) -> Option<*mut libc::c_void> {
        self.addr()
    }

    fn lineno(&self) -> Option<u32> {
        self.lineno()
    }

    fn filename(&self) -> Option<std::path::PathBuf> {
        self.filename().map(|filename| filename.to_owned())
    }
}

pub trait Frame: Sized + Clone {
    fn resolve_symbol<F: FnMut(Symbol)>(&self, cb: F);
    fn symbol_address(&self) -> *mut c_void;
    fn ip(&self) -> usize;
}

pub trait Trace {
    type Frame: Frame;

    fn trace<F: FnMut(&Self::Frame) -> bool>(_: *mut libc::c_void, cb: F)
    where
        Self: Sized;
}

#[cfg(not(all(
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ),
    any(feature = "frame-pointer", feature = "framehop-unwinder")
)))]
mod backtrace_rs;
#[cfg(not(all(
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ),
    any(feature = "frame-pointer", feature = "framehop-unwinder")
)))]
pub use backtrace_rs::Trace as TraceImpl;

#[cfg(all(
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ),
    feature = "frame-pointer"
))]
pub mod frame_pointer;
#[cfg(all(
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ),
    feature = "frame-pointer"
))]
pub use frame_pointer::Trace as TraceImpl;

#[cfg(all(
    any(target_arch = "x86_64", target_arch = "aarch64",),
    any(target_os = "linux", target_os = "macos",),
    feature = "framehop-unwinder"
))]
pub mod framehop_unwinder;

#[cfg(all(
    any(target_arch = "x86_64", target_arch = "aarch64",),
    any(target_os = "linux", target_os = "macos",),
    feature = "framehop-unwinder"
))]
pub use framehop_unwinder::Trace as TraceImpl;

#[cfg(all(
    any(target_arch = "x86_64", target_arch = "aarch64",),
    any(target_os = "linux", target_os = "macos",),
    feature = "framehop-unwinder"
))]
pub use framehop_unwinder::init_perfmap_resolver;
