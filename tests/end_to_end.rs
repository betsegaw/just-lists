use std::{fs, process::Command};

#[test]
fn golden_test() -> Result<(), String> {
    let check_vhs = Command::new("vhs")
        .arg("--version")
        .output()
        .expect("Required `vhs` command was not accessible.");

    assert!(check_vhs.status.success(), "vhs command is not available");
    
    let mut temp_folder = tempfile::tempdir().expect("Unable to create temporary directory.");
    _ = fs::copy("./docs/demo.tape", format!("{}/demo.tape",temp_folder.path().to_str().unwrap()));

    let run_demo_file = Command::new("vhs")
        .arg("demo.tape")
        .current_dir(temp_folder.path())
        .output();

    match run_demo_file {
        Ok(_) => {
            let old_golden_file = fs::read_to_string("./docs/golden.ascii").unwrap();
            let new_golden_file = fs::read_to_string(format!("{}/golden.ascii",temp_folder.path().to_str().unwrap())).unwrap();

            if old_golden_file != new_golden_file {
                return Err("Output of the golden scenario has changed. Please regenerate using `vhs demo.tape` if the changes are expected".to_string());
            }

            Ok(())
        },
        Err(e) => {
            Err(format!("Generating the run execution failed. Code: {}", e.raw_os_error().unwrap()))
        }
    }
}