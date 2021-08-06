use std::fs::File;
use std::io::{Read, BufReader};

fn same_files(f1: &mut File, f2: &mut File) -> bool {
    let mut buff1 = [0u8; 4096];
    let mut buff2 = [0u8; 4096];
    let mut reader1 = BufReader::new(f1);
    let mut reader2 = BufReader::new(f2);

    loop {
        match reader1.read(&mut buff1) {
            Err(_) => return false,
            Ok(f1_read_len) => match reader2.read(&mut buff2) {
                Err(_) => return false,
                Ok(f2_read_len) => {
                    if f1_read_len != f2_read_len {
                        return false;
                    }
                    if f1_read_len == 0 {
                        return true;
                    }
                    if buff1[0..f1_read_len] != buff2[0..f2_read_len] {
                        return false;
                    }
                }
            },
        }
    }
}

pub fn same(path1: &str, path2: &str) -> bool {
    let fh1 = File::open(path1);
    let fh2 = File::open(path2);

    match fh1 {
        Err(_) => false,
        Ok(mut file1) => match fh2 {
            Err(_) => false,
            Ok(mut file2) => same_files(&mut file1, &mut file2),
        },
    }
}
