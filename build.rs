use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    // Create the resource files first
    create_icon_resource();
    create_version_resource();

    // Compile the resources using your cross-compilation method
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        // Compile icon resource
        compile_embedded_resource("app_icon.rc", "app_icon.res");
        println!("cargo:rerun-if-changed=app_icon.rc");
        println!("cargo:rustc-link-arg-bin=self_delete-rs=app_icon.res"); // Replace your_binary_name

        // Compile version resource
        compile_embedded_resource("version.rc", "version.res");
        println!("cargo:rerun-if-changed=version.rc");
        println!("cargo:rustc-link-arg-bin=self_delete-rs=version.res"); // Replace your_binary_name
    }
}

fn create_icon_resource() {
    let mut rc_file = File::create("app_icon.rc").unwrap();
    writeln!(rc_file, "100 ICON \"icon/sheep-2-rb-256.ico\"").unwrap();
    println!("cargo:rerun-if-changed=icon/sheep-2-rb-256.ico");
}

fn create_version_resource() {
    let mut rc_file = File::create("version.rc").unwrap();

    writeln!(rc_file, "1 VERSIONINFO").unwrap();
    writeln!(rc_file, "FILEVERSION 1,0,0,0").unwrap();
    writeln!(rc_file, "PRODUCTVERSION 1,0,0,0").unwrap();
    writeln!(rc_file, "FILEFLAGSMASK 0x3fL").unwrap();
    writeln!(rc_file, "FILEFLAGS 0x0L").unwrap();
    writeln!(rc_file, "FILEOS 0x40004L").unwrap();
    writeln!(rc_file, "FILETYPE 0x1L").unwrap();
    writeln!(rc_file, "FILESUBTYPE 0x0L").unwrap();
    writeln!(rc_file, "BEGIN").unwrap();
    writeln!(rc_file, "    BLOCK \"StringFileInfo\"").unwrap();
    writeln!(rc_file, "    BEGIN").unwrap();
    writeln!(rc_file, "        BLOCK \"040904b0\"").unwrap();
    writeln!(rc_file, "        BEGIN").unwrap();
    writeln!(
        rc_file,
        "            VALUE \"CompanyName\", \"Your Company\\0\""
    )
    .unwrap();
    writeln!(
        rc_file,
        "            VALUE \"FileDescription\", \"Your Application\\0\""
    )
    .unwrap();
    writeln!(rc_file, "            VALUE \"FileVersion\", \"1.0.0.0\\0\"").unwrap();
    writeln!(
        rc_file,
        "            VALUE \"InternalName\", \"yourapp\\0\""
    )
    .unwrap();
    writeln!(
        rc_file,
        "            VALUE \"LegalCopyright\", \"Copyright © 2024\\0\""
    )
    .unwrap();
    writeln!(
        rc_file,
        "            VALUE \"OriginalFilename\", \"self_delete-rs.exe\\0\""
    )
    .unwrap();
    writeln!(
        rc_file,
        "            VALUE \"ProductName\", \"Your Product\\0\""
    )
    .unwrap();
    writeln!(
        rc_file,
        "            VALUE \"ProductVersion\", \"1.0.0.0\\0\""
    )
    .unwrap();
    writeln!(rc_file, "        END").unwrap();
    writeln!(rc_file, "    END").unwrap();
    writeln!(rc_file, "    BLOCK \"VarFileInfo\"").unwrap();
    writeln!(rc_file, "    BEGIN").unwrap();
    writeln!(rc_file, "        VALUE \"Translation\", 0x409, 1200").unwrap();
    writeln!(rc_file, "    END").unwrap();
    writeln!(rc_file, "END").unwrap();
}

/// Compiles a Windows resource file (.rc) into a .res file using windres
fn compile_embedded_resource(input: &str, output: &str) {
    // Ensure the input file exists
    if !Path::new(input).exists() {
        panic!("The file {} does not exist!", input);
    }

    // Execute the windres command
    let output_status = Command::new("x86_64-w64-mingw32-windres")
        .args(&[input, "-O", "coff", "-o", output])
        .output()
        .expect("Failed to execute windres");

    // Check if the command succeeded
    if !output_status.status.success() {
        panic!(
            "windres failed with status {}. Output: {}",
            output_status.status,
            String::from_utf8_lossy(&output_status.stderr)
        );
    }

    println!("Successfully compiled {} to {}", input, output);
}
