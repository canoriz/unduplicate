use clap::Parser;
use walkdir::WalkDir;

mod grouper;
use grouper::file_hash::{FastSamples, HashOption};
use grouper::FileList;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value_t = false)]
    list: bool,
    #[clap(short, long, value_parser, default_value_t = true)]
    info: bool,
    #[clap(short, long, value_parser, default_value_t = false)]
    bitwise: bool,
    #[clap(short, long, value_parser)]
    delete: bool,
    #[clap(value_parser)]
    dirs: Vec<String>,
}


fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    let dir_list: Vec<String> = match args.dirs.len() {
        0 => {
            vec![String::from("./")]
        }
        _ => {
            args.dirs.iter().map(|x| x.to_string()).collect()
        }
    };

    //let args: Vec<String> = env::args().skip(1).collect();

    let mut list = FileList::new();

    for path in dir_list {
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

    list.sort_by_path()
        .print_info(args.info, "none")
        .split_by_hash(HashOption::Length)
        .print_info(args.info, "length")
        //.split_by_hash(HashOption::Head(1))
        .split_by_hash(HashOption::Head(4))
        .print_info(args.info, "head 4*128 bytes")
        //.split_by_hash(HashOption::Head(16))
        //.split_by_hash(HashOption::Head(64))
        //.split_by_hash(HashOption::Head(256))
        .split_by_hash(HashOption::Fast(FastSamples::default()))
        .print_info(args.info, "eigen points")
        .split_by_hash(HashOption::Fnv(64))
        .print_info(args.info, "fnv hash 64*128 bytes")
        .split_by_hash(HashOption::FnvFull)
        .print_info(args.info, "fnv full file hash")
        .bitwise_compare(args.bitwise)
        .print_info(args.info & args.bitwise, "bitwise")
        .print_results(args.list)
        .delete_duplicates(args.delete);
}
