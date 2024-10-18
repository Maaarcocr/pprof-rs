use std::{io::BufRead, path::{Path, PathBuf}, sync::Arc, time::Duration};

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use notify_debouncer_mini::{new_debouncer, notify::{self, FsEventWatcher, RecursiveMode}, DebounceEventHandler, DebouncedEvent, DebouncedEventKind, Debouncer};

use crate::{backtrace::AsSymbol, Error};

pub struct PerfMap {
    ranges: Vec<(usize, usize, String)>,
}

impl PerfMap {
    pub fn new() -> Option<Self> {
        let path = PathBuf::from("/tmp/").join(format!("perf-{}.map", std::process::id()));
        let file = std::fs::File::open(&path).ok()?;
        let reader = std::io::BufReader::new(file);
        let mut ranges = Vec::new();
        for line in reader.lines() {
            let line = line.ok()?;
            // The format of perf map is:
            // <start addr> <len addr> <name>
            // where <start addr> and <len addr> are hexadecimal numbers.
            // where <name> may contain spaces.
            let mut parts = line.split_whitespace();
            let start = usize::from_str_radix(parts.next()?, 16).ok()?;
            let len = usize::from_str_radix(parts.next()?, 16).ok()?;
            let name = parts.collect::<Vec<_>>().join(" ");
            ranges.push((start, start + len, name));
        }
        Some(Self { ranges })
    }

    pub fn find(&self, addr: usize) -> Option<&str> {
        for (start, end, name) in &self.ranges {
            if *start <= addr && addr < *end {
                return Some(name);
            }
        }
        None
    }
}

pub struct PerfMapSymbol(String);

impl AsSymbol for PerfMapSymbol {
    fn name(&self) -> Option<Vec<u8>> {
        Some(self.0.as_bytes().to_vec())
    }

    fn addr(&self) -> Option<*mut std::ffi::c_void> {
        None
    }

    fn lineno(&self) -> Option<u32> {
        None
    }

    fn filename(&self) -> Option<PathBuf> {
        None
    }
}

pub struct PerfMapResolver {
    perf_map: Arc<Mutex<Option<PerfMap>>>
}

fn create_debouncer<F: DebounceEventHandler>(event_handler: F) -> Result<Debouncer<FsEventWatcher>, Error> {
    let mut debouncer = new_debouncer(Duration::from_secs(1), event_handler).map_err(|_| Error::CreatingError)?;
    let path = PathBuf::from("/tmp/").join(format!("perf-{}.map", std::process::id()));
    debouncer.watcher().watch(&path, RecursiveMode::NonRecursive).map_err(|_| Error::CreatingError)?;
    Ok(debouncer)
}

impl PerfMapResolver {
    pub fn new() -> Result<Self, Error> {
        let perf_map = Arc::new(Mutex::new(PerfMap::new()));
        let (tx, rx) = std::sync::mpsc::channel();

        let debouncer = create_debouncer(tx)?;
        let thread_perf_map = Arc::clone(&perf_map);

        std::thread::spawn(move || {
            let _debouncer = debouncer;
            for result in rx {
                match result {
                    Ok(_events) => {
                        let mut perf_map = thread_perf_map.lock();
                        *perf_map = PerfMap::new();  
                    }
                    Err(error) => log::info!("Error {error:?}"),
                }
            }
        });
        Ok(Self { perf_map })
    }

    pub fn resolve(&self, addr: usize) -> Option<PerfMapSymbol> {
        if let Some(perf_map) = self.perf_map.lock().as_ref() {
            perf_map.find(addr).map(|s| PerfMapSymbol(s.to_string()))
        } else {
            None
        }
    }
}

pub static PERF_MAP_RESOLVER: OnceCell<PerfMapResolver> = OnceCell::new();

pub fn init_perfmap_resolver() -> Result<(), Error>  {
    let perf_map_resolver = PerfMapResolver::new()?;
    PERF_MAP_RESOLVER.set(perf_map_resolver).map_err(|_| Error::CreatingError)
}