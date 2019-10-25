fn main() {
    tonic_build::compile_protos("proto/dapr.proto").unwrap();
    tonic_build::compile_protos("proto/daprclient.proto").unwrap();
}
