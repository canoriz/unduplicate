use std::fmt::Write;
use std::fs;
use std::io::{self, Read, Seek, BufReader};

#[derive(Eq, Debug, PartialEq)]
pub enum EigenOption {
    Fast(FastSamples),
    Head512,
}

#[derive(Eq, Debug, PartialEq)]
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

#[derive(Eq, Debug, PartialEq)]
pub enum FeatureResult {
    Fast([u8; 32]),
    Head512([u8, 64]),
}

impl FeatureResult {
    pub fn hex(&self) -> String {
        let mut res = String::from("");
        match self {
            FeatureResult::Fast(arr) => {
                for b in arr {
                    write!(&mut res, "{:02x}", b).expect("unable to write");
                }
            }
        }
        res
    }
}

type FileGroup = u64;
#[derive(PartialEq, Eq, Debug)]
pub struct FileInfo {
    path: String,
    len: u64,
    hash: FeatureResult,
    belongs: FileGroup,
}

impl FileInfo {
    pub fn new(path: String, belong: u64) -> Result<Self, io::Error> {
        let mut f = fs::File::open(&path)?;

        Ok(FileInfo {
            path: path,
            len: f.metadata()?.len(),
            hash: calc(&mut f, EigenOption::Fast(FastSamples::default()))?,
            belongs: belong,
        })
    }
}

pub fn calc(f: &mut fs::File, op: EigenOption) -> Result<FeatureResult, io::Error> {
    match op {
        EigenOption::Fast(FastSamples { samples, cuts }) => {
            let len = f.metadata()?.len();
            let bufchar = &mut [0u8; 1];
            let reader = BufReader::new(f);

            let mut extractor = |reader: &mut BufReader, sample_pos| -> Result<u8, io::Error> {
                reader.seek(io::SeekFrom::Start(len * sample_pos / cuts))?;
                reader.read(bufchar)?;
                Ok(bufchar[0])
            };

            let feature_vec = samples
                .iter()
                .map(|pos| extractor(reader, pos))
                .take(32)
                .collect::<Vec<Result<u8, io::Error>>>();
            let mut result: [u8; 32] = [0; 32];

            // convert vec to array
            let mut i = 0;
            for res in feature_vec {
                match res {
                    Ok(r) => result[i] = r,
                    Err(e) => return Err(e),
                }
                i += 1;
            }

            Ok(FeatureResult::Fast(result))
        }
        EigenOption::Head512 => {
            let mut reader = BufReader::new(f);
            let mut buf = [0u8; 512];
            match reader.read(&mut buf)? => {
                512 => (),
                n => for &mut b in buf.iter_mut().skip(n) {
                    b = 0u8;
                },
            }
            Ok(FeatureResult::Head512(buf))
        }
    }
}
