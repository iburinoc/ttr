fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/protos/")
        .inputs(&["protos/ttr.proto"])
        .run()
        .expect("protoc");
}
