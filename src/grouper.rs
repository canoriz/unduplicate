use std::cmp::Ordering;
use std::io;
use std::vec::Vec;

pub mod file_hash;
use file_hash::{EigenOption, FileInfo};

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
            if px < py {
                self.parent[py] = px;
            } else {
                self.parent[px] = py;
            }
        }
    }
}

type FileId = usize;
#[derive(Debug)]
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
    hash_option: EigenOption,
    files: Vec<FileInfo>,
}

impl FileList {
    pub fn new(h: EigenOption) -> Self {
        FileList {
            hash_option: h,
            files: Vec::<FileInfo>::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn add(&mut self, name: &str) -> Result<(), io::Error> {
        self.files.push(FileInfo::new(name, self.hash_option)?);
        Ok(())
    }

    fn sort_by_hash(&self) -> Vec<Vec<FileRecord>> {
        // comparer, first by len, then by hash
        let comparer = |a: &FileInfo, other: &FileInfo| match a.len.cmp(&other.len) {
            Ordering::Equal => a.hash.cmp(&other.hash),
            _ => a.len.cmp(&other.len),
        };

        let mut sorted = self.files.clone();
        sorted.sort_by(comparer);

        let mut same_hash_files = Vec::<Vec<FileRecord>>::new();
        let mut prev_file: Option<FileInfo> = None;

        let mut ingroup_id = 0;
        for file in sorted {
            let same_hash_and_len = |prev: &Option<FileInfo>, current: &FileInfo| match prev {
                None => false,
                Some(p) => p.hash == current.hash && p.len == current.len,
            };

            if !same_hash_and_len(&prev_file, &file) {
                ingroup_id = 0;
                prev_file = Some(file.clone());
                same_hash_files.push(Vec::<FileRecord>::new());
            }
            same_hash_files
                .last_mut()
                .unwrap()
                .push(FileRecord::new(file, ingroup_id));
            ingroup_id += 1;
        }
        same_hash_files
    }

    // pub fn compare_and_group(&self) -> Vec<Vec<FileRecord>> {
    pub fn compare_and_group(&self) -> Vec<Vec<FileInfo>> {
        let same_hash_files = self.sort_by_hash();

        println!("sort by hash ok");

        let split = |file_group: &mut Vec<FileRecord>| {
            let mut merger = Merger::new(file_group.len());
            /*
            file_group
                .iter()
                .enumerate()
                .filter(|(index1, file1)| merger.belongs(file1.id) == file1.id)
                .map(|(index1, file1)| {
                    file_group
                        .iter()
                        .skip(index1 + 1)
                        .filter(|file2| merger.belongs(file2.id) == file2.id)
                        .map(|file2| {
                            if same(&file1.info.path, &file2.info.path) {
                                merger.merge(file1.id, file2.id);
                            }
                        })
                        .collect::<()>();
                })
                .collect::<()>();
            */

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

                        println!("compare {}, {}", &file1.info.path, &file2.info.path);

                        if same(&file2.info.path, &file1.info.path) {
                            // merges two sub sets
                            merger.merge(file1.id, file2.id);
                        }
                    }
                }
            }

            // group files to list
            let mut output_list = Vec::<Vec<FileInfo>>::new();
            file_group.sort_by_key(|x| merger.belongs(x.id));
            let mut prev_id: Option<usize> = None;
            for file in file_group {
                match prev_id {
                    Some(grp) => {
                        if grp != merger.belongs(file.id) {
                            output_list.push(Vec::<FileInfo>::new())
                        }
                    }
                    None => output_list.push(Vec::<FileInfo>::new()),
                }
                output_list.last_mut().unwrap().push(file.info.clone());
                prev_id = Some(merger.belongs(file.id));
            }

            output_list
        };

        same_hash_files
            .into_iter()
            .filter(|file_group| file_group.len() > 1)
            .map(|mut file_group| split(&mut file_group))
            .flatten()
            .collect()
    }

    pub fn list_same_files(&self) {
        let grouped = self.compare_and_group();

        for same_file_group in grouped {
            if same_file_group.len() > 1 {
                println!();
                for file_info in same_file_group {
                    println!("{}", file_info.path);
                }
            }
        }
    }
}
