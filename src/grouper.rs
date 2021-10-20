use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use std::{fs, io};

pub mod file_hash;
use file_hash::{FileInfo, HashOption};

mod file_diff;
use file_diff::same;

struct Merger {
    // union find struct
    // currently support objects of type can convert to usize
    parent: Vec<usize>,
}

impl Merger {
    fn new<T: Into<usize>>(size: T) -> Self {
        Merger {
            parent: (0..size.into()).collect(),
        }
    }

    fn belongs(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.belongs(self.parent[x]);
        }
        self.parent[x]
    }

    fn merge(&mut self, x: usize, y: usize) {
        let (px, py) = (self.belongs(x), self.belongs(y));
        if px != py {
            self.parent[py] = px;
        }
    }
}

type FileId = usize;
#[derive(Debug, Clone)]
struct FileRecord {
    info: FileInfo,
    id: FileId,
}

impl FileRecord {
    fn new(finfo: FileInfo, fid: FileId) -> Self {
        FileRecord {
            info: finfo,
            id: fid,
        }
    }
}

#[derive(Debug)]
pub struct FileList {
    files: Vec<Vec<FileRecord>>,
}

fn human_size(mut size: f32) -> String {
    let units = vec!["B", "KiB", "MiB", "GiB", "TiB"];
    for (_, unit) in units.iter().enumerate().filter(|(idx, _)| *idx < 4) {
        if size < 1024.0 {
            return format!("{:.3} {}", size, unit);
        }
        size /= 1024.0;
    }
    return format!("{} {}", size, units[4]);
}

impl FileList {
    pub fn new() -> Self {
        FileList {
            files: Vec::<Vec<FileRecord>>::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn add(&mut self, name: &str) -> Result<(), io::Error> {
        if self.files.is_empty() {
            self.files.push(Vec::<FileRecord>::new());
        }
        let current_id = self.files[0].len();
        self.files[0].push(FileRecord::new(FileInfo::new(name)?, current_id));
        Ok(())
    }

    fn remove_unique(self) -> Self {
        FileList {
            files: self
                .files
                .into_iter()
                .filter(|group| group.len() > 1)
                .collect(),
        }
    }

    pub fn sort_by_path(mut self) -> Self {
        self.files.iter_mut().for_each(|file_group| {
            file_group.sort_by(|a: &FileRecord, b: &FileRecord| a.info.path.cmp(&b.info.path))
        });
        self.files
            .sort_by(|a: &Vec<FileRecord>, b: &Vec<FileRecord>| {
                a[0].info.path.cmp(&b[0].info.path)
            });
        self
    }

    fn make_hash(mut self, hash_option: HashOption) -> Self {
        self.files.par_iter_mut().for_each(|file_group| {
            file_group.par_iter_mut().for_each(|file| {
                file.info.calc_hash(hash_option);
            })
        });
        FileList { files: self.files }
    }

    pub fn split_by_hash(self, hash_option: HashOption) -> Self {
        let split = |mut file_group: Vec<FileRecord>| {
            file_group
                .sort_by(|a: &FileRecord, other: &FileRecord| a.info.hash.cmp(&other.info.hash));
            let mut same_hash_files = Vec::<Vec<FileRecord>>::new();
            let mut prev_file: Option<FileInfo> = None;

            let mut ingroup_id = 0;

            file_group
                .iter()
                .map(|record| record.info.clone())
                // records have same hash with previos hashing round
                // split them
                .for_each(|file| {
                    let same_hash = |prev: &Option<FileInfo>, current: &FileInfo| match prev {
                        None => false,
                        Some(p) => p.hash == current.hash,
                    };

                    if !same_hash(&prev_file, &file) {
                        ingroup_id = 0;
                        prev_file = Some(file.clone());
                        same_hash_files.push(Vec::<FileRecord>::new());
                    }
                    same_hash_files
                        .last_mut()
                        .unwrap()
                        .push(FileRecord::new(file, ingroup_id));
                    ingroup_id += 1;
                });
            same_hash_files
        };

        let hashed = self.make_hash(hash_option);

        FileList {
            files: hashed
                .files
                .into_iter()
                .filter(|file_group| file_group.len() > 1)
                .map(split)
                .flatten()
                .collect(),
        }
        .remove_unique()
    }

    pub fn bitwise_compare(self) -> Self {
        let split = |file_group: &mut Vec<FileRecord>| {
            let merger = Arc::new(Mutex::new(Merger::new(file_group.len())));

            for (index1, file1) in file_group.iter().enumerate() {
                if merger.lock().unwrap().belongs(file1.id) != file1.id {
                    // file1 is same with previous file, skip it
                    continue;
                }
                // file1 is not same with any previous file
                // see if any file same with file1
                file_group.iter().skip(index1 + 1).for_each(|file2| {
                    if merger.lock().unwrap().belongs(file2.id) == file2.id {
                        // file2 is unique file

                        if same(&file2.info.path, &file1.info.path) {
                            // merges two sub sets
                            merger.lock().unwrap().merge(file1.id, file2.id);
                        }
                    }
                });
            }

            // group files to list
            let mut output_list = Vec::<Vec<FileRecord>>::new();
            file_group.sort_by_key(|x| merger.lock().unwrap().belongs(x.id));
            let mut prev_id: Option<usize> = None;
            for file in file_group {
                match prev_id {
                    Some(grp) => {
                        if grp != merger.lock().unwrap().belongs(file.id) {
                            output_list.push(Vec::<FileRecord>::new())
                        }
                    }
                    None => output_list.push(Vec::<FileRecord>::new()),
                }
                let current_id = output_list.last_mut().unwrap().len();
                output_list
                    .last_mut()
                    .unwrap()
                    .push(FileRecord::new(file.info.clone(), current_id));
                prev_id = Some(merger.lock().unwrap().belongs(file.id));
            }

            output_list
        };

        FileList {
            files: self
                .files
                .into_iter()
                .map(|mut file_group| split(&mut file_group))
                .flatten()
                .collect(),
        }
        .remove_unique()
    }

    pub fn delete_duplicates(self, delete_flag: bool) {
        if delete_flag {
            let delete = |file_group: Vec<FileRecord>| {
                println!();
                file_group
                    .iter()
                    .skip(1)
                    .for_each(|frecord| {
                        println!("Delete {}", &frecord.info.path);
                        fs::remove_file(&frecord.info.path).unwrap_or(())
                    });
            };

            self.files.into_iter().for_each(delete);
        }
    }

    pub fn print_results(self, print_flag: bool) -> Self {
        if print_flag {
            for same_file_group in &self.files {
                if same_file_group.len() > 1 {
                    println!();
                    for file_record in same_file_group {
                        println!("{}", file_record.info.path);
                    }
                }
            }
        }
        self
    }

    pub fn print_info(self, info_flag: bool, method: &str) -> Self {
        if info_flag {
            println!("Grouping using [{}]", method);
            println!(
                "  {} group {} candidates takes {} ",
                self.files.len(),
                self.files.iter().map(|s| s.len()).sum::<usize>(),
                human_size(
                    self.files
                        .iter()
                        .map(|s| s.iter().skip(1).map(|f| f.info.len).sum::<u64>())
                        .sum::<u64>() as f32
                )
            );
        }
        self
    }
}
