# self_delete-rs

# Self-deletion running executable from disk in Rust

A Rust implementation that allows a running Windows executable to delete itself from disk using Windows API calls. This technique works around file locking mechanisms by leveraging Alternate Data Streams (ADS) and file disposition flags.

## Compatibility

- **Windows 10, Windows 11 (pre-24H2)**: Uses traditional `FILE_DISPOSITION_INFO` method
- **Windows 11 24H2 and later**: Uses new `FILE_DISPOSITION_INFO_EX` method with `FILE_DISPOSITION_FLAG_POSIX_SEMANTICS` to comply with enhanced security restrictions (tested on 25H2)
- **Cross-compilation**: Supports building from macOS/Linux to Windows targets

## Technical Overview

The code uses Windows API bindings to achieve self-deletion through a multi-step process:

### 1. File Renaming via Alternate Data Stream
- Retrieves the path of the current executable using `GetModuleFileNameW`
- Creates a file handle with `DELETE` and `SYNCHRONIZE` access rights
- Renames the file to an Alternate Data Stream (e.g., `:backtomato`) using `SetFileInformationByHandle` with `FILE_RENAME_INFO`
- This breaks the file locking while keeping the executable running

### 2. File Deletion with Version-Specific Methods
**For Windows 11 24H2 and later:**
- Uses `FILE_DISPOSITION_INFO_EX` structure
- Applies `FILE_DISPOSITION_FLAG_DELETE | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS` flags
- Required due to security enhancements in Windows 24H2

**For older Windows versions:**
- Uses traditional `FILE_DISPOSITION_INFO` structure
- Sets `DeleteFile = TRUE` for immediate deletion

### 3. Fallback Mechanism
The implementation includes automatic detection and fallback:
- Attempts the new Windows 24H2 method first
- Falls back to traditional method if the new API is unavailable
- Handles both scenarios gracefully without user intervention

## Code Example
