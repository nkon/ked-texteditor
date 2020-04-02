use std::fs;
use std::process::Command;

#[test]
fn find_dir_and_run() {
    println!("macro_tests_runner.rs");
    
    let target = "./tests/script/";
    for entry in fs::read_dir(target).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let mut scr_file = path.display().to_string();
            scr_file.push_str("/run.sh");
            let status = Command::new(&scr_file).status().unwrap();
            assert!(status.success());
        }
    }
}

