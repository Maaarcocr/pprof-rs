use crate::Symbol;

impl super::Frame for backtrace::Frame {
    fn ip(&self) -> usize {
        self.ip() as usize
    }

    fn resolve_symbol<F: FnMut(Symbol)>(&self, mut cb: F) {
        backtrace::resolve_frame(self, |s| cb(Symbol::from(s)));
    }

    fn symbol_address(&self) -> *mut libc::c_void {
        self.symbol_address()
    }
}

pub struct Trace {}

impl super::Trace for Trace {
    type Frame = backtrace::Frame;

    fn trace<F: FnMut(&Self::Frame) -> bool>(_: *mut libc::c_void, cb: F) {
        unsafe { backtrace::trace_unsynchronized(cb) }
    }
}

