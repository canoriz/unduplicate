use std::fmt::Write;
use std::fs;
use std::io::{self, BufReader, Read, Seek};

#[derive(Eq, Debug, PartialEq, Copy, Clone)]
pub enum EigenOption {
    Fast(FastSamples),
    Head,
}

#[derive(Eq, Debug, PartialEq, Copy, Clone)]
pub struct FastSamples {
    samples: [u64; 32],
    cuts: u64,
}

impl Default for FastSamples {
    fn default() -> Self {
        // the default sampling positions, generated by random numbers
        FastSamples {
            samples: [
                697, 378, 107, 428, 427, 626, 774, 501, 776, 692, 233, 760, 66, 131, 68, 118, 992,
                362, 436, 354, 980, 932, 686, 869, 474, 313, 432, 746, 1009, 611, 454, 681,
            ],
            cuts: 1024,
        }
    }
}

#[derive(Eq, Debug, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum HashResult {
    Fast([u8; 32]),
    Head([u8; 64]),
}

impl HashResult {
    pub fn hex(&self) -> String {
        let mut res = String::from("");
        match self {
            HashResult::Fast(arr) => {
                for b in arr {
                    write!(&mut res, "{:02x}", b).expect("unable to write");
                }
            }
            HashResult::Head(arr) => {
                for b in arr {
                    write!(&mut res, "{:02x}", b).expect("unable to write");
                }
            }
        }
        res
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub len: u64,
    pub hash: HashResult,
}

impl FileInfo {
    pub fn new(path: &str, e: EigenOption) -> Result<Self, io::Error> {
        let mut f = fs::File::open(path)?;

        println!("building hash for {}", path);

        Ok(FileInfo {
            path: path.to_string(),
            len: f.metadata()?.len(),
            hash: FileInfo::calc_feature(&mut f, e)?,
        })
    }

    fn calc_feature(f: &mut fs::File, op: EigenOption) -> Result<HashResult, io::Error> {
        match op {
            EigenOption::Fast(FastSamples { samples, cuts }) => {
                let len = f.metadata()?.len();
                let mut reader = BufReader::new(f);
                let bufchar = &mut [0u8; 1];

                let mut extractor =
                    |reader: &mut BufReader<&mut fs::File>, sample_pos| -> Result<u8, io::Error> {
                        reader.seek(io::SeekFrom::Start(len * sample_pos / cuts))?;
                        reader.read_exact(bufchar)?;
                        Ok(bufchar[0])
                    };

                let feature_vec = samples
                    .iter()
                    .map(|pos| extractor(&mut reader, pos))
                    .take(32);
                let mut result: [u8; 32] = [0; 32];

                // convert vec to array
                for (i, res) in feature_vec.into_iter().enumerate() {
                    match res {
                        Ok(r) => result[i] = r,
                        Err(e) => return Err(e),
                    }
                }

                Ok(HashResult::Fast(result))
            }
            EigenOption::Head => {
                let mut result = [0u8; 64];
                let mut reader = BufReader::new(f);

                match reader.read(&mut result)? {
                    64 => (),
                    x => {
                        for b in result.iter_mut().skip(x) {
                            *b = 0u8;
                        }
                    }
                }
                Ok(HashResult::Head(result))
            }
        }
    }
}
