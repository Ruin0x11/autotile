pub fn read_string(path: &str) -> String {
    use std::io::Read;
    use std::fs::File;

    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}
