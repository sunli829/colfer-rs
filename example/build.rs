use colfer_build::Config;

fn main() {
    Config::new()
        .out_dir("./src")
        .compile(&["test.colf"])
        .unwrap();
}
