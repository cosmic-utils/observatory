fn main() {
    let paths = std::fs::read_dir("/proc").unwrap();
    for path in paths {
        println!("{}", path.unwrap().path().display());
    }
}
