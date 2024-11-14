use lazy_static::lazy_static;
use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{
    SymCleanup, SymFromNameW, SymInitializeW, SymLoadModuleExW, SYMBOL_INFOW, SYM_LOAD_FLAGS,
};
use windows::Win32::System::ProcessStatus::EnumDeviceDrivers;
use windows::Win32::System::Threading::GetCurrentProcess;

lazy_static! {
    static ref KERNEL_BASE: Option<u64> = {
        let r = get_krn_addr();
        if r.is_err() {
            return None;
        }

        return Some(r.unwrap());
    };
}

pub fn get_krn_addr() -> windows::core::Result<u64> {
    let mut krnl_addr: *mut c_void = null_mut();
    let mut ret_len = 0;

    let result = unsafe {
        EnumDeviceDrivers(
            &mut krnl_addr as *mut _,
            std::mem::size_of::<*mut usize>() as u32,
            &mut ret_len,
        )
    };

    if result.is_err() {
        return Err(windows::core::Error::from_win32());
    }

    if !krnl_addr.is_null() {
        Ok(krnl_addr as u64)
    } else {
        return Err(windows::core::Error::from_win32());
    }
}

pub struct SymbolLoader {
    process_handle: HANDLE,
}

impl SymbolLoader {
    pub fn new(symbol_path: Option<&str>) -> windows::core::Result<Self> {
        unsafe {
            let mut path =
                String::from("srv*C:\\Symbols*https://msdl.microsoft.com/download/symbols");
            if symbol_path.is_some() {
                let symbol_path = symbol_path.unwrap();
                path = format!(
                    "srv*{}*https://msdl.microsoft.com/download/symbols",
                    &symbol_path
                );
            }

            let process_handle = GetCurrentProcess();
            let sym_path = to_wide_string(&path);

            SymInitializeW(process_handle, PCWSTR(sym_path.as_ptr()), true)?;

            Ok(SymbolLoader { process_handle })
        }
    }

    pub fn load_module(&self, module_path: &str) -> windows::core::Result<u64> {
        let module_path = to_wide_string(module_path);
        unsafe {
            let base_address = SymLoadModuleExW(
                self.process_handle,
                HANDLE::default(),
                PCWSTR(module_path.as_ptr()),
                PCWSTR::null(),
                0,
                0,
                None,
                SYM_LOAD_FLAGS(0),
            );

            if base_address != 0 {
                Ok(base_address)
            } else {
                Err(windows::core::Error::from_win32())
            }
        }
    }

    pub fn get_symbol_address(&self, symbol_name: &str) -> windows::core::Result<u64> {
        let symbol_name = to_wide_string(symbol_name);
        let mut symbol_info: SYMBOL_INFOW = SYMBOL_INFOW {
            SizeOfStruct: std::mem::size_of::<SYMBOL_INFOW>() as u32,
            MaxNameLen: 255,
            ..Default::default()
        };

        unsafe {
            SymFromNameW(
                self.process_handle,
                PCWSTR(symbol_name.as_ptr()),
                &mut symbol_info,
            )?;

            Ok(symbol_info.Address)
        }
    }

    pub fn get_kernel_symbol_address(&self, symbol_name: &str) -> windows::core::Result<u64> {
        let module_base = self.load_module("C:\\Windows\\System32\\ntoskrnl.exe")?;

        let symbol_address = self.get_symbol_address(symbol_name)?;

        if KERNEL_BASE.is_none() {
            return Err(windows::core::Error::from_win32());
        }

        let r = symbol_address - module_base + KERNEL_BASE.unwrap();
        Ok(r)
    }
}

impl Drop for SymbolLoader {
    fn drop(&mut self) {
        unsafe {
            let _ = SymCleanup(self.process_handle);
        }
    }
}

fn to_wide_string(s: &str) -> Vec<u16> {
    OsString::from(s)
        .encode_wide()
        .chain(std::iter::once(0)) // Null-terminator
        .collect()
}
