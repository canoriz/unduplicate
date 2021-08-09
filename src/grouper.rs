use std::io;
use std::vec::Vec;

pub mod file_hash;
use file_hash::{HashOption, FileInfo};

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

    fn make_hash(mut self, hash_option: HashOption) -> Self {
        for file_group in self.files.iter_mut() {
            for file in file_group {
                file.info.calc_hash(hash_option);
            }
        }
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

        println!("split using hash {:?}", hash_option);
        println!(
            "before: {} group {} candidates",
            self.files.len(),
            self.files.iter().map(|s| s.len()).sum::<usize>()
        );
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
            let mut merger = Merger::new(file_group.len());

            for (index1, file1) in file_group.iter().enumerate() {
                if merger.belongs(file1.id) != file1.id {
                    // file1 is same with previous file, skip it
                    continue;
                }
                // file1 is not same with any previous file
                // see if any file same with file1
                for file2 in file_group.iter().skip(index1 + 1) {
                    if merger.belongs(file2.id) == file2.id {
                        // file2 is unique file

                        if same(&file2.info.path, &file1.info.path) {
                            // merges two sub sets
                            merger.merge(file1.id, file2.id);
                        }
                    }
                }
            }

            // group files to list
            let mut output_list = Vec::<Vec<FileRecord>>::new();
            file_group.sort_by_key(|x| merger.belongs(x.id));
            let mut prev_id: Option<usize> = None;
            for file in file_group {
                match prev_id {
                    Some(grp) => {
                        if grp != merger.belongs(file.id) {
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
                prev_id = Some(merger.belongs(file.id));
            }

            output_list
        };

        println!("bitwise compare");
        println!(
            "before: {} group {} candidates",
            self.files.len(),
            self.files.iter().map(|s| s.len()).sum::<usize>()
        );

        FileList {
            files: self.files
            .into_iter()
            .map(|mut file_group| split(&mut file_group))
            .flatten()
            .collect(),
        }.remove_unique()
    }

    pub fn print_results(&self) {
        for same_file_group in &self.files {
            if same_file_group.len() > 1 {
                println!();
                for file_record in same_file_group {
                    println!("{}", file_record.info.path);
                }
            }
        }
    }
}
