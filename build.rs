
fn main(){
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["grpc-protobufs/account/account_service.proto"],
            &["grpc-protobufs/account"]
        )
        .unwrap();
}