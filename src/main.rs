use same_file::is_same_file;
use std::fs::File;
use std::io;

mod file_feature;

/*
fn try_main() -> Result<(), io::Error> {
    assert!(is_same_file("/tmp/rstest", "/tmp/rstest2")?);
    Ok(())
}

fn main() {
    // try_main().unwrap();
    let result = file_feature::calc(
        &mut File::open("Cargo.toml").unwrap(),
        file_feature::EigenOption::Fast(file_feature::FastSamples::default()),
    )
    .unwrap();
    println!("feature: {:?}", result);
    println!("feature: {}", result.hex());
}
*/

use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = String::from(entry.path().to_string_lossy());
        let f_meta = entry.metadata();

        let result = file_feature::calc(
            &mut File::open(f_name).unwrap(),
            file_feature::EigenOption::Fast(file_feature::FastSamples::default()),
            )
            .unwrap();
        println!("feature: {:?}", result);
        println!("feature: {}", result.hex());
    }
}
