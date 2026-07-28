#![cfg(feature = "full")]
use libloading::{Library, Symbol};

macro_rules! uc16 {
    ($s:expr) => {{
        let mut v: Vec<u16> = $s.encode_utf16().collect();
        v.push(0);
        v
    }};
}

type NTSTATUS = i32;
const STATUS_SUCCESS: i32 = 0;

pub struct Bcrypt {
    lib: Library,
    sha256: usize,
    sha512: usize,
}

unsafe impl Send for Bcrypt {}
unsafe impl Sync for Bcrypt {}

impl Bcrypt {
    pub fn load() -> Result<Self, String> {
        let lib = unsafe { Library::new("bcrypt.dll") }
            .map_err(|e| format!("bcrypt.dll: {e}"))?;

        unsafe {
            let open: Symbol<unsafe extern "C" fn(*mut usize, *const u16, *const u16, i32) -> NTSTATUS> =
                lib.get(b"BCryptOpenAlgorithmProvider\0")
                    .map_err(|e| format!("BCryptOpen: {e}"))?;

            let mut sha256 = 0usize;
            let mut sha512 = 0usize;

            let s = uc16!("SHA256");
            let s2 = uc16!("SHA512");

            if open(&mut sha256, s.as_ptr(), std::ptr::null(), 0) != STATUS_SUCCESS {
                return Err("BCryptOpen SHA256 failed".into());
            }
            if open(&mut sha512, s2.as_ptr(), std::ptr::null(), 0) != STATUS_SUCCESS {
                return Err("BCryptOpen SHA512 failed".into());
            }

            Ok(Bcrypt { lib, sha256, sha512 })
        }
    }

    pub fn sha256(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            let h: Symbol<unsafe extern "C" fn(usize, *mut u8, i32, *const u8, i32, *mut u8, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptHash\0").map_err(|e| e.to_string())?;
            let mut hash = vec![0u8; 32];
            if h(self.sha256, std::ptr::null_mut(), 0, data.as_ptr(), data.len() as i32, hash.as_mut_ptr(), 32) != STATUS_SUCCESS {
                return Err("BCryptHash SHA256 fail".into());
            }
            Ok(hash)
        }
    }

    pub fn sha512(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            let h: Symbol<unsafe extern "C" fn(usize, *mut u8, i32, *const u8, i32, *mut u8, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptHash\0").map_err(|e| e.to_string())?;
            let mut hash = vec![0u8; 64];
            if h(self.sha512, std::ptr::null_mut(), 0, data.as_ptr(), data.len() as i32, hash.as_mut_ptr(), 64) != STATUS_SUCCESS {
                return Err("BCryptHash SHA512 fail".into());
            }
            Ok(hash)
        }
    }
}
