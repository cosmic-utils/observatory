fn main() {
    let paths = std::path::Path::new("/proc");
    for path in paths {
        println!("{}", path);
    }
}
