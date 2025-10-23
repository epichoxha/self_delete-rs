use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::prelude::OsStrExt;
use std::ptr::{copy_nonoverlapping, null};
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError};
use windows_sys::Win32::Storage::FileSystem::CreateFileW;
use windows_sys::Win32::Storage::FileSystem::{
    DELETE, FILE_ATTRIBUTE_NORMAL, FILE_DISPOSITION_INFO, FILE_RENAME_INFO, OPEN_EXISTING,
    SetFileInformationByHandle,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;

const MAX: usize = 500usize;
const FILE_DISPOSITION_FLAG_DELETE: u32 = 0x00000001;
const FILE_DISPOSITION_FLAG_POSIX_SEMANTICS: u32 = 0x00000002;

// Define FILE_DISPOSITION_INFO_EX structure
#[repr(C)]
struct FileDispositionInfoEx {
    flags: u32,
}

pub unsafe fn handle_file_operation() {
    let mut buffer: [u16; MAX + 1] = [0; MAX + 1];

    unsafe {
        if !get_module_file_name(&mut buffer) {
            println!("[!] Unable to retrieve module file name.");
            return;
        }
    }

    let current_handle = unsafe { create_file(buffer.as_ptr()) };

    if current_handle == 0 {
        let error_code = unsafe { GetLastError() };
        println!(
            "[!] Failed to open current file handle. Error code: {}",
            error_code
        );
        return;
    }

    unsafe {
        if !set_file_rename_info(
            current_handle,
            &OsStr::new(":12").encode_wide().collect::<Vec<_>>(),
        ) {
            let error_code = GetLastError();
            CloseHandle(current_handle);
            println!(
                "[!] Failed to set file rename info. Error code: {}",
                error_code
            );
            return;
        }
    }

    unsafe {
        CloseHandle(current_handle);
    }

    let mut buffer2: [u16; MAX + 1] = [0; MAX + 1];

    unsafe {
        if !get_module_file_name(&mut buffer2) {
            println!("[!] Unable to retrieve module file name after rename.");
            return;
        }
    }

    let new_handle = unsafe { create_file(buffer2.as_ptr()) };

    if new_handle == -1 {
        let error_code = unsafe { GetLastError() };
        println!(
            "[!] Failed to open new file handle. Error code: {}",
            error_code
        );
        return;
    }

    unsafe {
        if !set_file_disposition_info_ex(new_handle) {
            let error_code = GetLastError();
            CloseHandle(new_handle);
            println!(
                "[!] Failed to set file disposition info. Error code: {}",
                error_code
            );
            return;
        }
    }

    unsafe {
        CloseHandle(new_handle);
    }

    println!("[+] File operation completed successfully.");
}

unsafe fn get_module_file_name(buffer: &mut [u16; MAX + 1]) -> bool {
    unsafe {
        let current_path = GetModuleFileNameW(0, buffer.as_mut_ptr(), 261u32);
        current_path != 0
    }
}

unsafe fn create_file(file_name: *const u16) -> isize {
    unsafe {
        let handle = CreateFileW(
            file_name,
            DELETE,
            0,
            null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            0,
        );
        if handle == -1 {
            let error_code = GetLastError();
            println!(
                "[!] Failed to create file handle. Error code: {}",
                error_code
            );
            return 0;
        }
        handle
    }
}

unsafe fn set_file_rename_info(handle: isize, file_name: &[u16]) -> bool {
    unsafe {
        let mut f_rename: FILE_RENAME_INFO = zeroed();
        f_rename.FileNameLength = (file_name.len() * size_of::<u16>()) as u32;
        copy_nonoverlapping(
            file_name.as_ptr(),
            f_rename.FileName.as_mut_ptr(),
            file_name.len(),
        );

        let rename_result = SetFileInformationByHandle(
            handle,
            3, // FileRenameInfo
            &f_rename as *const FILE_RENAME_INFO as *mut _,
            size_of::<FILE_RENAME_INFO>() as u32,
        );
        if rename_result == 0 {
            let error_code = GetLastError();
            println!(
                "[!] Failed to set file rename info. Error code: {}",
                error_code
            );
        }
        rename_result != 0
    }
}

unsafe fn set_file_disposition_info_ex(handle: isize) -> bool {
    unsafe {
        let mut temp: FileDispositionInfoEx = zeroed();
        temp.flags = FILE_DISPOSITION_FLAG_DELETE | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS;

        let disp_result = SetFileInformationByHandle(
            handle,
            21, // FileDispositionInfoEx - Windows 24H2
            &temp as *const FileDispositionInfoEx as *mut _,
            size_of::<FileDispositionInfoEx>() as u32,
        );
        if disp_result == 0 {
            let error_code = GetLastError();
            println!(
                "[!] Failed to set file disposition info (EX). Error code: {}",
                error_code
            );

            // Fallback to old method if EX fails
            println!("[!] Trying fallback to old FILE_DISPOSITION_INFO...");
            return set_file_disposition_info_old(handle);
        }
        disp_result != 0
    }
}

// Keep the old method as fallback
unsafe fn set_file_disposition_info_old(handle: isize) -> bool {
    unsafe {
        let mut temp: FILE_DISPOSITION_INFO = zeroed();
        temp.DeleteFile = 1;

        let disp_result = SetFileInformationByHandle(
            handle,
            4, // FileDispositionInfo
            &temp as *const FILE_DISPOSITION_INFO as *mut _,
            size_of::<FILE_DISPOSITION_INFO>() as u32,
        );
        if disp_result == 0 {
            let error_code = GetLastError();
            println!(
                "[!] Failed to set file disposition info (old). Error code: {}",
                error_code
            );
        }
        disp_result != 0
    }
}

fn main() {
    unsafe {
        handle_file_operation();
    }
}
