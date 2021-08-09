use std::env;

use walkdir::WalkDir;

mod grouper;
use grouper::file_hash::{HashOption, FastSamples};
use grouper::FileList;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut list = FileList::new();

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

    list
        .split_by_hash(HashOption::Length)
        .split_by_hash(HashOption::Head(1))
        .split_by_hash(HashOption::Head(4))
        .split_by_hash(HashOption::Head(16))
        .split_by_hash(HashOption::Head(64))
        .split_by_hash(HashOption::Head(256))
        .split_by_hash(HashOption::Fast(FastSamples::default()))
        .bitwise_compare()
        .print_results();
}
