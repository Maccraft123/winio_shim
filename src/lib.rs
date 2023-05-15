use windows::core::PWSTR;
use std::ptr;
use std::io::Write;
use std::net::TcpStream;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct tag_phys_struct {
    phys_mem_size_in_bytes: u64,
    phys_address: u64,
    memory_handle: u64,
    mem_lin: u64,
    phys_section: u64,
}

static OUT_TCP: OnceCell<Mutex<TcpStream>> = OnceCell::new();

#[allow(unused_must_use)]
fn log(what: String) {
    let mut stream = OUT_TCP.get_or_init(|| Mutex::new(TcpStream::connect("192.168.1.101:2137").unwrap()))
        .lock()
        .unwrap();
    stream.write_all(what.as_str().as_bytes());
    stream.write_all(b"\n\0");
}

#[no_mangle]
pub extern "C" fn MapPhysToLin(_data: *mut tag_phys_struct) -> *const u8 {
    ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn UnmapPhysicalMemory(data: *mut tag_phys_struct) -> bool {
    if data.is_null() {
        log(format!("MapPhysicalMemory NULL[NULL]"));
    } else {
        let addr = (*data).phys_address;
        let size = (*data).phys_mem_size_in_bytes;
        log(format!("MapPhysicalMemory {:x}[{:x}]", addr, size));
    }
    true
}

#[no_mangle]
pub extern "C" fn GetPortVal(addr: u16, val: *mut u32, size: u8) -> bool { 
    let ret = actual_io::get_port_val(addr, val, size);
    log(format!("Get {:x} {:x}", addr, unsafe {*val}));
    ret
}

#[no_mangle]
pub extern "C" fn SetPortVal(addr: u16, val: u32, size: u8) -> bool {
    log(format!("Set {:x} {:x}", addr, val));
    actual_io::set_port_val(addr, val, size)
}

/* unused in ayaspace */
#[no_mangle] pub extern "C" fn GetPhysLong(_addr: *mut u8, _val: *mut u32) -> bool { false }
#[no_mangle] pub extern "C" fn SetPhysLong(_addr: *mut u8, _val: u32) -> bool { false }
#[no_mangle] pub extern "C" fn InitializeWinIo() -> bool {
    actual_io::init()
}
#[no_mangle] pub extern "C" fn ShutdownWinIo() {}
#[no_mangle] pub extern "C" fn InstallWinIoDriver(_path: PWSTR, _demand_loaded: bool) -> bool { false }
#[no_mangle] pub extern "C" fn RemoveWinIoDriver() {}

mod actual_io {
    use libloading::{Symbol, Library};
    use once_cell::sync::Lazy;
    static WINIO: Lazy<Library> = Lazy::new(|| {
        unsafe {
            Library::new("actual_winio64.dll").unwrap()
        }
    });
    fn get_lib() -> &'static Library {
        &WINIO
    }
    pub fn get_port_val(addr: u16, val: *mut u32, size: u8) -> bool {
        unsafe {
            let func: Symbol<unsafe extern "C" fn (u16, *mut u32, u8) -> bool> = get_lib().get(b"GetPortVal\0").unwrap();
            func(addr, val, size)
        }
    }
    pub fn set_port_val(addr: u16, val: u32, size: u8) -> bool {
        unsafe {
            let func: Symbol<unsafe extern "C" fn (u16, u32, u8) -> bool> = get_lib().get(b"SetPortVal\0").unwrap();
            func(addr, val, size)
        }
    }
    pub fn init() -> bool {
        unsafe {
            let func: Symbol<unsafe extern "C" fn () -> bool> = get_lib().get(b"InitializeWinIo\0").unwrap();
            func()
        }
    }
}
