#![cfg(target_os = "windows")]

use std::mem::{zeroed, size_of};
use std::ptr::{null_mut, copy_nonoverlapping};
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows_sys::Win32::Storage::FileSystem::{
    DELETE, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, SetFileInformationByHandle,
    FILE_RENAME_INFO, FILE_DISPOSITION_INFO,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::Storage::FileSystem::CreateFileW;

const MAX_PATH_LENGTH: usize = 500;
const FILE_DISPOSITION_FLAG_DELETE: u32 = 0x00000001;
const FILE_DISPOSITION_FLAG_POSIX_SEMANTICS: u32 = 0x00000002;

#[repr(C)]
struct FileDispositionInfoEx {
    flags: u32,
}

#[derive(Debug)]
pub enum SelfDeleteError {
    ModulePathNotFound,
    FileHandleFailed(u32),
    RenameFailed(u32),
    DispositionFailed(u32),
}

impl std::fmt::Display for SelfDeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelfDeleteError::ModulePathNotFound => write!(f, "Could not retrieve module file path"),
            SelfDeleteError::FileHandleFailed(code) => write!(f, "Failed to open file handle (error: {})", code),
            SelfDeleteError::RenameFailed(code) => write!(f, "Failed to rename file (error: {})", code),
            SelfDeleteError::DispositionFailed(code) => write!(f, "Failed to set file disposition (error: {})", code),
        }
    }
}

impl std::error::Error for SelfDeleteError {}

/// Safe wrapper for Windows file handle
struct FileHandle(HANDLE);

impl FileHandle {
    /// Opens a file with DELETE access rights
    fn open_for_deletion(path: &[u16]) -> Result<Self, SelfDeleteError> {
        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                DELETE,
                0,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            )
        };

        if handle.is_null() {
            let error_code = unsafe { GetLastError() };
            Err(SelfDeleteError::FileHandleFailed(error_code))
        } else {
            Ok(FileHandle(handle))
        }
    }

    /// Renames the file to an alternate data stream - using EXACT same approach as working code
    fn rename_to_stream(&self) -> Result<(), SelfDeleteError> {
        // Use the exact same stream name and encoding as the working code
        let stream_name = ":12";
        let stream_wide: Vec<u16> = stream_name.encode_utf16().collect(); // No null terminator

        unsafe {
            let mut rename_info: FILE_RENAME_INFO = zeroed();
            // Use the exact same calculation as working code
            rename_info.FileNameLength = (stream_wide.len() * size_of::<u16>()) as u32;

            copy_nonoverlapping(
                stream_wide.as_ptr(),
                rename_info.FileName.as_mut_ptr(),
                stream_wide.len(),
            );

            let result = SetFileInformationByHandle(
                self.0,
                3, // FileRenameInfo
                &rename_info as *const _ as *mut _,
                size_of::<FILE_RENAME_INFO>() as u32,
            );

            if result == 0 {
                let error_code = GetLastError();
                Err(SelfDeleteError::RenameFailed(error_code))
            } else {
                Ok(())
            }
        }
    }

    /// Marks the file for deletion using the appropriate method for the Windows version
    fn mark_for_deletion(&self) -> Result<(), SelfDeleteError> {
        // Try the new Windows 24H2 method first
        match self.mark_for_deletion_ex() {
            Ok(()) => {
                println!("[+] Used FILE_DISPOSITION_INFO_EX method (Windows 24H2+)");
                Ok(())
            }
            Err(_) => {
                println!("[-] FILE_DISPOSITION_INFO_EX failed, falling back to traditional method");
                self.mark_for_deletion_old()
            }
        }
    }

    fn mark_for_deletion_ex(&self) -> Result<(), SelfDeleteError> {
        unsafe {
            let disposition_info = FileDispositionInfoEx {
                flags: FILE_DISPOSITION_FLAG_DELETE | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS,
            };

            let result = SetFileInformationByHandle(
                self.0,
                21, // FileDispositionInfoEx
                &disposition_info as *const _ as *mut _,
                size_of::<FileDispositionInfoEx>() as u32,
            );

            if result == 0 {
                let error_code = GetLastError();
                Err(SelfDeleteError::DispositionFailed(error_code))
            } else {
                Ok(())
            }
        }
    }

    fn mark_for_deletion_old(&self) -> Result<(), SelfDeleteError> {
        unsafe {
            let mut disposition_info: FILE_DISPOSITION_INFO = zeroed();
            disposition_info.DeleteFile = true;

            let result = SetFileInformationByHandle(
                self.0,
                4, // FileDispositionInfo
                &disposition_info as *const _ as *mut _,
                size_of::<FILE_DISPOSITION_INFO>() as u32,
            );

            if result == 0 {
                let error_code = GetLastError();
                Err(SelfDeleteError::DispositionFailed(error_code))
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

/// Gets the current executable path as a wide character string
fn get_current_executable_path() -> Result<[u16; MAX_PATH_LENGTH + 1], SelfDeleteError> {
    let mut buffer = [0u16; MAX_PATH_LENGTH + 1];

    let path_length = unsafe {
        GetModuleFileNameW(null_mut(), buffer.as_mut_ptr(), MAX_PATH_LENGTH as u32)
    };

    if path_length == 0 {
        Err(SelfDeleteError::ModulePathNotFound)
    } else {
        Ok(buffer)
    }
}

/// Performs the self-deletion operation
pub fn self_delete() -> Result<(), SelfDeleteError> {
    println!("[+] Starting self-deletion process...");

    let original_path = get_current_executable_path()?;
    println!("[+] Current executable path retrieved");

    // Step 1: Rename the file to break the lock
    println!("[+] Step 1: Renaming file to alternate data stream...");
    {
        let file_handle = FileHandle::open_for_deletion(&original_path)?;
        file_handle.rename_to_stream()?; // Use the exact same approach
        println!("[+] File renamed successfully");
        // FileHandle is automatically closed here due to Drop trait
    }

    // Small delay to ensure rename is processed
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 2: Reopen and mark for deletion
    println!("[+] Step 2: Marking file for deletion...");
    let new_path = get_current_executable_path()?;
    let file_handle = FileHandle::open_for_deletion(&new_path)?;
    file_handle.mark_for_deletion()?;
    println!("[+] File marked for deletion successfully");

    // FileHandle automatically closed here
    Ok(())
}

fn main() {
    match self_delete() {
        Ok(()) => {
            println!("[+] Self-deletion completed successfully!");
            println!("[+] File will be deleted when this process exits");
            // Keep process alive for a bit to demonstrate it still runs
            println!("[+] Process continues running for 5 seconds...");
            std::thread::sleep(std::time::Duration::from_secs(5));
            println!("[+] Process exiting, file should now be deleted");
        }
        Err(e) => {
            println!("[-] Self-deletion failed: {}", e);
            println!("[-] This might be due to Windows version restrictions or permissions");
            std::process::exit(1);
        }
    }
}