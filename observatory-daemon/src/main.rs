use std::io::Write;

fn main() {
    let paths = std::fs::read_dir("/proc").unwrap();
    let mut out = std::fs::File::create("/home/adamc/Desktop/processes.txt").unwrap();

    for path in paths {
        out.write_all(format!("{}", path.unwrap().path().display()).as_bytes()).unwrap();
    }
}
