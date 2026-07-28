#![cfg(feature = "full")]
use libloading::{Library, Symbol};
use std::ffi::c_void;

macro_rules! uc16 {
    ($s:expr) => {{
        let mut v: Vec<u16> = $s.encode_utf16().collect();
        v.push(0);
        v
    }};
}

type NTSTATUS = i32;
const STATUS_SUCCESS: i32 = 0;
const STATUS_BUFFER_TOO_SMALL: i32 = 0xC0000023u32 as i32;

pub struct Bcrypt {
    lib: Library,
    sha256: usize,
    sha512: usize,
    aes: usize,
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
            let mut aes = 0usize;

            let s = uc16!("SHA256");
            let s2 = uc16!("SHA512");
            let s_aes = uc16!("AES");

            if open(&mut sha256, s.as_ptr(), std::ptr::null(), 0) != STATUS_SUCCESS {
                return Err("BCryptOpen SHA256 failed".into());
            }
            if open(&mut sha512, s2.as_ptr(), std::ptr::null(), 0) != STATUS_SUCCESS {
                return Err("BCryptOpen SHA512 failed".into());
            }
            if open(&mut aes, s_aes.as_ptr(), std::ptr::null(), 0) != STATUS_SUCCESS {
                return Err("BCryptOpen AES failed".into());
            }

            Ok(Bcrypt { lib, sha256, sha512, aes })
        }
    }

    pub fn aes_encrypt(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            // BCryptEncrypt signature:
            // NTSTATUS BCryptEncrypt(hKey, pbInput, cbInput, pPaddingInfo, pbIV, cbIV, pbOutput, cbOutput, pcbResult, dwFlags)
            let encrypt: Symbol<unsafe extern "C" fn(usize, *const u8, i32, *mut c_void, *mut u8, i32, *mut u8, i32, *mut i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptEncrypt\0").map_err(|e| e.to_string())?;

            // BCryptGenerateSymmetricKey signature:
            // NTSTATUS BCryptGenerateSymmetricKey(hAlgorithm, phKey, pbKeyObject, cbKeyObject, pbSecret, cbSecret, dwFlags)
            let gen_key: Symbol<unsafe extern "C" fn(usize, *mut usize, *mut u8, i32, *const u8, i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptGenerateSymmetricKey\0").map_err(|e| e.to_string())?;

            // BCryptDestroyKey signature
            let destroy: Symbol<unsafe extern "C" fn(usize, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptDestroyKey\0").map_err(|e| e.to_string())?;

            // Set chaining mode to CBC via BCryptSetProperty
            let set_prop: Symbol<unsafe extern "C" fn(usize, *const u16, *const u8, i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptSetProperty\0").map_err(|e| e.to_string())?;
            let chain_mode = uc16!("ChainingMode");
            let cbc = uc16!("ChainingModeCBC");
            if set_prop(self.aes, chain_mode.as_ptr(), cbc.as_ptr() as *const u8, (cbc.len() * 2) as i32, 0) != STATUS_SUCCESS {
                return Err("BCryptSetProperty CBC failed".into());
            }

            // Get key object size via BCryptGetProperty
            let get_prop: Symbol<unsafe extern "C" fn(usize, *const u16, *mut u8, i32, *mut i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptGetProperty\0").map_err(|e| e.to_string())?;
            let obj_len_prop = uc16!("ObjectLength");
            let mut obj_len = 0i32;
            let mut result_len = 0i32;
            if get_prop(self.aes, obj_len_prop.as_ptr(), &mut obj_len as *mut i32 as *mut u8, 4, &mut result_len, 0) != STATUS_SUCCESS {
                return Err("BCryptGetProperty ObjectLength failed".into());
            }

            // Allocate key object and generate key handle
            let mut key_obj = vec![0u8; obj_len as usize];
            let mut key_handle: usize = 0;
            if gen_key(self.aes, &mut key_handle, key_obj.as_mut_ptr(), obj_len, key.as_ptr(), key.len() as i32, 0) != STATUS_SUCCESS {
                return Err("BCryptGenerateSymmetricKey failed".into());
            }

            // PKCS7 padding
            let block_size = 16usize;
            let pad_len = block_size - (data.len() % block_size);
            let mut padded = data.to_vec();
            padded.resize(data.len() + pad_len, pad_len as u8);

            let mut iv = vec![0u8; 16]; // zero IV

            // First call to get output size (pbOutput=NULL, cbOutput=0)
            let mut cipher_len = 0i32;
            let status = encrypt(
                key_handle,
                padded.as_ptr(),
                padded.len() as i32,
                std::ptr::null_mut(),
                iv.as_mut_ptr(),
                iv.len() as i32,
                std::ptr::null_mut(),
                0,
                &mut cipher_len,
                0,
            );

            if status != STATUS_SUCCESS && status != STATUS_BUFFER_TOO_SMALL {
                destroy(key_handle, 0);
                return Err(format!("BCryptEncrypt size query failed: {}", status));
            }

            let mut ciphertext = vec![0u8; cipher_len as usize];
            let mut written = 0i32;
            let status = encrypt(
                key_handle,
                padded.as_ptr(),
                padded.len() as i32,
                std::ptr::null_mut(),
                iv.as_mut_ptr(),
                iv.len() as i32,
                ciphertext.as_mut_ptr(),
                cipher_len,
                &mut written,
                0,
            );

            destroy(key_handle, 0);

            if status != STATUS_SUCCESS {
                return Err(format!("BCryptEncrypt failed: {}", status));
            }

            ciphertext.truncate(written as usize);
            Ok(ciphertext)
        }
    }

    pub fn aes_decrypt(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            let decrypt: Symbol<unsafe extern "C" fn(usize, *const u8, i32, *mut c_void, *mut u8, i32, *mut u8, i32, *mut i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptDecrypt\0").map_err(|e| e.to_string())?;

            let gen_key: Symbol<unsafe extern "C" fn(usize, *mut usize, *mut u8, i32, *const u8, i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptGenerateSymmetricKey\0").map_err(|e| e.to_string())?;

            let destroy: Symbol<unsafe extern "C" fn(usize, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptDestroyKey\0").map_err(|e| e.to_string())?;

            let set_prop: Symbol<unsafe extern "C" fn(usize, *const u16, *const u8, i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptSetProperty\0").map_err(|e| e.to_string())?;
            let chain_mode = uc16!("ChainingMode");
            let cbc = uc16!("ChainingModeCBC");
            if set_prop(self.aes, chain_mode.as_ptr(), cbc.as_ptr() as *const u8, (cbc.len() * 2) as i32, 0) != STATUS_SUCCESS {
                return Err("BCryptSetProperty CBC failed".into());
            }

            let get_prop: Symbol<unsafe extern "C" fn(usize, *const u16, *mut u8, i32, *mut i32, i32) -> NTSTATUS> =
                self.lib.get(b"BCryptGetProperty\0").map_err(|e| e.to_string())?;
            let obj_len_prop = uc16!("ObjectLength");
            let mut obj_len = 0i32;
            let mut result_len = 0i32;
            if get_prop(self.aes, obj_len_prop.as_ptr(), &mut obj_len as *mut i32 as *mut u8, 4, &mut result_len, 0) != STATUS_SUCCESS {
                return Err("BCryptGetProperty ObjectLength failed".into());
            }

            let mut key_obj = vec![0u8; obj_len as usize];
            let mut key_handle: usize = 0;
            if gen_key(self.aes, &mut key_handle, key_obj.as_mut_ptr(), obj_len, key.as_ptr(), key.len() as i32, 0) != STATUS_SUCCESS {
                return Err("BCryptGenerateSymmetricKey failed".into());
            }

            let mut iv = vec![0u8; 16];
            let mut plain_len = 0i32;
            let status = decrypt(
                key_handle,
                data.as_ptr(),
                data.len() as i32,
                std::ptr::null_mut(),
                iv.as_mut_ptr(),
                iv.len() as i32,
                std::ptr::null_mut(),
                0,
                &mut plain_len,
                0,
            );

            if status != STATUS_SUCCESS && status != STATUS_BUFFER_TOO_SMALL {
                destroy(key_handle, 0);
                return Err(format!("BCryptDecrypt size query failed: {}", status));
            }

            let mut plaintext = vec![0u8; plain_len as usize];
            let mut written = 0i32;
            let status = decrypt(
                key_handle,
                data.as_ptr(),
                data.len() as i32,
                std::ptr::null_mut(),
                iv.as_mut_ptr(),
                iv.len() as i32,
                plaintext.as_mut_ptr(),
                plain_len,
                &mut written,
                0,
            );

            destroy(key_handle, 0);

            if status != STATUS_SUCCESS {
                return Err(format!("BCryptDecrypt failed: {}", status));
            }

            // Remove PKCS7 padding
            if written > 0 && written as usize <= plaintext.len() {
                let pad_byte = plaintext[(written - 1) as usize];
                let pad_len = pad_byte as usize;
                if pad_len > 0 && pad_len <= 16 && pad_len <= written as usize {
                    plaintext.truncate((written - pad_len as i32) as usize);
                } else {
                    plaintext.truncate(written as usize);
                }
            }

            Ok(plaintext)
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
