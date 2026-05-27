use crate::core::product::CREDENTIAL_TARGET_PREFIX;

pub struct CredentialStore;

impl CredentialStore {
    pub fn save_tool_token(tool_id: i32, token: &str) -> Result<(), String> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return Err("Credential token is empty.".to_string());
        }

        platform::save(&Self::target_name(tool_id), trimmed)
    }

    pub fn save_provider_token(credential_ref: &str, token: &str) -> Result<(), String> {
        let trimmed_ref = credential_ref.trim();
        if trimmed_ref.is_empty() {
            return Err("Provider credential reference is empty.".to_string());
        }
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return Err("Credential token is empty.".to_string());
        }

        platform::save(&Self::provider_target_name(trimmed_ref), trimmed)
    }

    pub fn load_provider_token(credential_ref: &str) -> Result<Option<String>, String> {
        let trimmed_ref = credential_ref.trim();
        if trimmed_ref.is_empty() {
            return Ok(None);
        }

        platform::load(&Self::provider_target_name(trimmed_ref))
    }

    pub fn clear_tool_token(tool_id: i32) -> Result<(), String> {
        platform::clear(&Self::target_name(tool_id))
    }

    pub fn clear_provider_token(credential_ref: &str) -> Result<(), String> {
        let trimmed_ref = credential_ref.trim();
        if trimmed_ref.is_empty() {
            return Ok(());
        }
        platform::clear(&Self::provider_target_name(trimmed_ref))
    }

    fn target_name(tool_id: i32) -> String {
        format!("{CREDENTIAL_TARGET_PREFIX}/{tool_id}/credential")
    }

    fn provider_target_name(credential_ref: &str) -> String {
        format!("{CREDENTIAL_TARGET_PREFIX}/provider/{credential_ref}/credential")
    }
}

#[cfg(windows)]
mod platform {
    use crate::core::product::PRODUCT_NAME;
    use std::ffi::{c_void, OsStr};
    use std::iter;
    use std::os::windows::ffi::OsStrExt;
    use std::slice;

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
    const ERROR_NOT_FOUND: u32 = 1168;

    #[repr(C)]
    struct FileTime {
        low_date_time: u32,
        high_date_time: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        type_: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: FileTime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut std::ffi::c_void,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    #[link(name = "Advapi32")]
    extern "system" {
        fn CredWriteW(credential: *const CredentialW, flags: u32) -> i32;
        fn CredReadW(
            target_name: *const u16,
            type_: u32,
            flags: u32,
            credential: *mut *mut CredentialW,
        ) -> i32;
        fn CredDeleteW(target_name: *const u16, type_: u32, flags: u32) -> i32;
        fn CredFree(buffer: *mut c_void);
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn GetLastError() -> u32;
    }

    pub fn save(target_name: &str, token: &str) -> Result<(), String> {
        let mut target = wide_null(target_name);
        let mut user_name = wide_null(PRODUCT_NAME);
        let mut blob = token.as_bytes().to_vec();

        let credential = CredentialW {
            flags: 0,
            type_: CRED_TYPE_GENERIC,
            target_name: target.as_mut_ptr(),
            comment: std::ptr::null_mut(),
            last_written: FileTime {
                low_date_time: 0,
                high_date_time: 0,
            },
            credential_blob_size: blob.len() as u32,
            credential_blob: blob.as_mut_ptr(),
            persist: CRED_PERSIST_LOCAL_MACHINE,
            attribute_count: 0,
            attributes: std::ptr::null_mut(),
            target_alias: std::ptr::null_mut(),
            user_name: user_name.as_mut_ptr(),
        };

        let ok = unsafe { CredWriteW(&credential, 0) };
        if ok == 0 {
            return Err(format!(
                "Failed to save credential to Windows Credential Manager. error={}",
                last_error()
            ));
        }

        Ok(())
    }

    pub fn clear(target_name: &str) -> Result<(), String> {
        let target = wide_null(target_name);
        let ok = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };

        if ok == 0 {
            let error = last_error();
            if error == ERROR_NOT_FOUND {
                return Ok(());
            }

            return Err(format!(
                "Failed to clear credential from Windows Credential Manager. error={error}"
            ));
        }

        Ok(())
    }

    pub fn load(target_name: &str) -> Result<Option<String>, String> {
        let target = wide_null(target_name);
        let mut credential: *mut CredentialW = std::ptr::null_mut();
        let ok = unsafe {
            CredReadW(
                target.as_ptr(),
                CRED_TYPE_GENERIC,
                0,
                &mut credential as *mut _,
            )
        };

        if ok == 0 {
            let error = last_error();
            if error == ERROR_NOT_FOUND {
                return Ok(None);
            }

            return Err(format!(
                "Failed to read credential from Windows Credential Manager. error={error}"
            ));
        }

        if credential.is_null() {
            return Ok(None);
        }

        let result = unsafe {
            let credential_ref = &*credential;
            let bytes = slice::from_raw_parts(
                credential_ref.credential_blob,
                credential_ref.credential_blob_size as usize,
            );
            String::from_utf8(bytes.to_vec())
                .map(Some)
                .map_err(|error| format!("Stored credential is not valid UTF-8: {error}"))
        };

        unsafe { CredFree(credential.cast::<c_void>()) };
        result
    }

    fn wide_null(value: &str) -> Vec<u16> {
        OsStr::new(value)
            .encode_wide()
            .chain(iter::once(0))
            .collect()
    }

    fn last_error() -> u32 {
        unsafe { GetLastError() }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn save(_target_name: &str, _token: &str) -> Result<(), String> {
        Err("System credential storage is only implemented for Windows in this build.".to_string())
    }

    pub fn clear(_target_name: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn load(_target_name: &str) -> Result<Option<String>, String> {
        Err("System credential storage is only implemented for Windows in this build.".to_string())
    }
}
