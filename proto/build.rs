fn main() {
    prost_build::compile_protos(&["../proto/pbv2.proto"], &["../proto/"])
        .expect("prost_build failed");
}
