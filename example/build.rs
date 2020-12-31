fn main() {
    colfer_build::Config::default()
        .out_dir("./src")
        .compile(&["test.colf", "bench.colf"])
        .unwrap();

    prost_build::Config::default()
        .out_dir("./src")
        .compile_protos(&["bench.proto"], &["./"])
        .unwrap();
}
