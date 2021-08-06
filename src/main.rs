use std::env;

use walkdir::WalkDir;

mod grouper;
use grouper::file_hash::{self, EigenOption, FastSamples};
use grouper::FileList;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut list = FileList::new(EigenOption::Fast(FastSamples::default()));
    // let mut list = FileList::new(EigenOption::Head);

    for path in args {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.metadata().is_ok())
            .filter(|e| e.metadata().unwrap().len() > 0)
        {
            let f_name = String::from(entry.path().to_string_lossy());

            match list.add(&f_name) {
                Ok(_) => (),
                Err(_) => continue,
            }
        }
    }

    list.compare_and_group();
    list.list_same_files();
}
