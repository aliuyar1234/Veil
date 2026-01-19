use std::path::Path;

pub(crate) fn harden_audit_log_file(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Best-effort: on Unix we explicitly set 0600 even if the file already existed.
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }

    #[cfg(windows)]
    {
        harden_windows_dacl(path);
    }
}

#[cfg(windows)]
fn harden_windows_dacl(path: &Path) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::{
        SetFileSecurityW, DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
        PSECURITY_DESCRIPTOR,
    };

    // Restrictive DACL: full access for SYSTEM, Administrators, and the file OWNER.
    // This approximates Unix 0600 semantics while still allowing system administration.
    //
    // SDDL references:
    // - SY: Local System
    // - BA: Built-in Administrators
    // - OW: Owner rights
    const SDDL: &str = "D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;OW)";

    let sddl_w: Vec<u16> = OsStr::new(SDDL).encode_wide().chain([0]).collect();

    let mut sd_ptr: PSECURITY_DESCRIPTOR = ptr::null_mut();
    let mut sd_len = 0u32;

    let ok = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl_w.as_ptr(),
            SDDL_REVISION_1,
            &mut sd_ptr,
            &mut sd_len,
        )
    };
    if ok == 0 || sd_ptr.is_null() {
        return;
    }

    let path_w: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let _ = unsafe {
        SetFileSecurityW(
            path_w.as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            sd_ptr,
        )
    };

    unsafe {
        let _ = LocalFree(sd_ptr);
    }
}
