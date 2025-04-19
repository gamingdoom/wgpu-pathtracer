#[macro_export]
macro_rules !preprocess_wgsl {
    ($file:expr $(,)?) => {{
        
        // Get executable path
        let exec_path = std::env::current_exe().unwrap();

        let f = exec_path.parent().unwrap().join($file).display().to_string();

        println!("Preprocessing {}.", f);

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("cat {} | sed 's/\\/\\/!//g' - | cpp -P -I{} -", f, std::path::Path::new(&f).parent().unwrap().display()))
            .output()
            .expect("Failed to Execute Preprocessor (are deps installed?)");

        if output.status.success() {
            std::str::from_utf8(&output.stdout).unwrap().to_string()
        } else {
            panic!("Preprocessor Failed! {}", String::from_utf8_lossy(&output.stderr));
        }
    }};
}