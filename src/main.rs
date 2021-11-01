use clap::{App, Arg};
use walkdir::WalkDir;

mod grouper;
use grouper::file_hash::{FastSamples, HashOption};
use grouper::FileList;

// flag options
#[derive(Default, Debug)]
struct Flags {
    delete: bool,
    list: bool,
    info: bool,
    bitwise: bool,
}

fn main() {
    let matches = App::new("unduplicate")
        .version("0.1")
        .author("yhc")
        .about("Find duplicate files")
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("List duplicate files"),
        )
        .arg(
            Arg::with_name("delete")
                .short("d")
                .long("delete")
                .help("Auto delete duplicate files"),
        )
        .arg(
            Arg::with_name("bitwise")
                .short("b")
                .long("bitwise")
                .help("Bitwise compare two file instead of hashing"),
        )
        .arg(
            Arg::with_name("info")
                .short("i")
                .long("info")
                .help("Print grouping infomation"),
        )
        .arg(Arg::with_name("dirs").help("Directories").multiple(true))
        .get_matches();

    let mut flags = Flags::default();
    let dir_list: Vec<_>;

    if matches.is_present("delete") {
        println!("delete!");
        flags.delete = true;
    }

    if matches.is_present("list") {
        println!("list!");
        flags.list = true;
    }

    if matches.is_present("info") {
        println!("info!");
        flags.info = true;
    }

    if matches.is_present("bitwise") {
        println!("bitwise!");
        flags.bitwise = true;
    }

    println!("{:?}", flags);

    if let Some(dirs) = matches.values_of("dirs") {
        dir_list = dirs.collect();
    } else {
        dir_list = vec!["./"];
    }

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
        .print_info(flags.info, "none")
        .split_by_hash(HashOption::Length)
        .print_info(flags.info, "length")
        //.split_by_hash(HashOption::Head(1))
        .split_by_hash(HashOption::Head(4))
        .print_info(flags.info, "head 4*128 bytes")
        //.split_by_hash(HashOption::Head(16))
        //.split_by_hash(HashOption::Head(64))
        //.split_by_hash(HashOption::Head(256))
        .split_by_hash(HashOption::Fast(FastSamples::default()))
        .print_info(flags.info, "eigen points")
        .split_by_hash(HashOption::Fnv(64))
        .print_info(flags.info, "fnv hash 64*128 bytes")
        .split_by_hash(HashOption::FnvFull)
        .print_info(flags.info, "fnv full file hash")
        .bitwise_compare(flags.bitwise)
        .print_info(flags.info & flags.bitwise, "bitwise")
        .print_results(flags.list)
        .delete_duplicates(flags.delete);
}
