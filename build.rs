use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Specify the source directory and the destination directory
    let destination_dir = out_dir();

    // Copy the directory recursively
    copy_dir_all("res", &destination_dir.join("res")).expect("Failed to copy resources directory");
    copy_dir_all("src/shaders", &destination_dir.join("shaders")).expect("Failed to copy shaders directory");
}

// Function to get the output directory
fn out_dir() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = std::env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
    return PathBuf::from(path);
}

// Function to copy a directory recursively
fn copy_dir_all<P: AsRef<Path>>(src: P, dst: &Path) -> std::io::Result<()> {
    let src = src.as_ref();
    if !src.is_dir() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Source is not a directory"));
    }

    // Create the destination directory
    fs::create_dir_all(dst)?;

    // Iterate over the entries in the source directory
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        // Construct the destination path
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            // Recursively copy subdirectories
            copy_dir_all(entry.path(), &dest_path)?;
        } else {
            // Preprocess files ending with .wgsl or .rs and then write it to the destination, otherwise copy
            if entry.path().extension().unwrap() == "wgsl" || entry.path().extension().unwrap() == "rs" {
                let content = wgsl_preprocessor::preprocess_wgsl!(fs::canonicalize(entry.path())?.display().to_string());
                
                //println!("cargo::warning={}", content);

                fs::write(&dest_path, content)?;

                // Have it be recompiled if changed
                println!("cargo:rerun-if-changed={}", entry.path().display());
            } else {
                // Copy the file
                fs::copy(entry.path(), &dest_path)?;
            }

        }
    }

    Ok(())
}