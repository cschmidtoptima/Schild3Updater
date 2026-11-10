use anyhow::{bail, Context, Result};
use semver::Version;
use std::path::Path;

pub const CLIENT_EXE: &str = "SchILD30.exe";

#[cfg(windows)]
pub fn read_version(path: &Path) -> Result<Version> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt};
    #[link(name = "version")]
    extern "system" {
        fn GetFileVersionInfoSizeW(filename: *const u16, handle: *mut u32) -> u32;
        fn GetFileVersionInfoW(
            filename: *const u16,
            handle: u32,
            length: u32,
            data: *mut c_void,
        ) -> i32;
        fn VerQueryValueW(
            data: *const c_void,
            key: *const u16,
            value: *mut *mut c_void,
            length: *mut u32,
        ) -> i32;
    }
    let filename: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut handle = 0;
    let length = unsafe { GetFileVersionInfoSizeW(filename.as_ptr(), &mut handle) };
    if length == 0 {
        bail!(
            "Keine Windows-Dateiversion lesbar: {} ({})",
            path.display(),
            std::io::Error::last_os_error()
        );
    }
    let mut buffer = vec![0u8; length as usize];
    let root = [92u16, 0];
    let mut value = std::ptr::null_mut();
    let mut value_length = 0;
    unsafe {
        if GetFileVersionInfoW(filename.as_ptr(), 0, length, buffer.as_mut_ptr().cast()) == 0
            || VerQueryValueW(
                buffer.as_ptr().cast(),
                root.as_ptr(),
                &mut value,
                &mut value_length,
            ) == 0
            || value_length < 52
            || value.is_null()
        {
            bail!(
                "Dateiversion konnte nicht gelesen werden: {}",
                path.display()
            );
        }
        let fields = value.cast::<u32>();
        if fields.read_unaligned() != 0xFEEF04BD {
            bail!("Ungültige Windows-Versionsinformation.");
        }
        let high = fields.add(2).read_unaligned();
        let low = fields.add(3).read_unaligned();
        Ok(Version::new(
            (high >> 16) as u64,
            (high & 0xFFFF) as u64,
            (low >> 16) as u64,
        ))
    }
}

#[cfg(not(windows))]
pub fn read_version(_path: &Path) -> Result<Version> {
    bail!("Windows-Dateiversionen können nur unter Windows gelesen werden.")
}

pub fn client_version(directory: &Path) -> Result<Version> {
    read_version(&directory.join(CLIENT_EXE))
        .context("Bitte einen vorhandenen SchILD3-Clientordner auswählen.")
}
