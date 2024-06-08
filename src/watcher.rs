use alloc::{string::{FromUtf8Error, String}, vec::Vec};
use asr::{future::IntoOption, settings::Value, watcher::Pair, Address, Error, Process};
use bytemuck::Pod;

pub struct Watcher<T> {
    path: Vec<u64>,
    pub current: T,
    pub old: T,
}

impl<T: Pod + PartialEq + PartialOrd> Watcher<T> {
    pub fn new(path: Vec<u64>, default: T) -> Self {
        Self {
            path,
            current: default,
            old: default,
        }
    }

    pub fn update(&mut self, process: &Process, module: u64) {
        self.old = self.current;
        self.current = process
            .read_pointer_path::<T>(module, asr::PointerSize::Bit64, &self.path)
            .unwrap_or(self.current);
    }

    pub fn changed(&self) -> bool {
        self.current != self.old
    }

    pub fn decreased(&self) -> bool {
        self.current < self.old
    }

    pub fn increased(&self) -> bool {
        self.current > self.old
    }

    pub fn changed_from_to(&self, from: T, to: T) -> bool {
        self.old == from && self.current == to
    }
}

pub struct StringWatcher {
    path: Vec<u64>,
    pub current: String,
    pub old: String,
}

impl StringWatcher {
    pub fn new(path: Vec<u64>) -> Self {
        Self {
            path,
            current: String::new(),
            old: String::new(),
        }
    }

    pub fn update(&mut self, process: &Process, module: u64) {
        self.old = self.current.clone();
        self.current = process
            .read_pointer_path::<u64>(module, asr::PointerSize::Bit64, &self.path)
            .and_then(|ptr| {
                let mut buf= [0; 255];
                process.read_into_buf(ptr, &mut buf).map(|_| buf)
            })
            .map_err(|_| ())
            .and_then(|bytes| bytes_to_string(&bytes).map_err(|_|()))
            .unwrap_or(self.current.clone());
    }

    pub fn changed(&self) -> bool {
        self.old != self.current
    }
}

fn bytes_to_string(utf8_src: &[u8]) -> Result<String, FromUtf8Error> {
    let nul_range_end = utf8_src.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len());
    String::from_utf8(utf8_src[0..nul_range_end].to_vec())
}
