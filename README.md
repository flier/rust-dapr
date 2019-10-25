# Dapr SDK for Rust

Dapr is a portable, event-driven, serverless runtime for building distributed applications across cloud and edge.

This is the Dapr SDK for Rust, based on the auto-generated protobuf client.

For more info on Dapr and gRPC, visit [this link](https://github.com/dapr/docs/tree/master/howto/create-grpc-app).

## Features

- Dapr [Runtime API](https://github.com/dapr/docs/tree/master/reference/api) with `dapr::Runtime`
- Dapr Client with `#[dapr::service]`

## Getting Started

### Rust Version

`dapr` currently works on rust `1.39-beta` and above as it requires support for the `async_await` feature. To install the beta simply follow the commands below:

```sh
$ rustup install beta
$ rustup component add rustfmt --toolchain beta
$ cargo +beta build
```

### Tutorials

The [examples](https://github.com/flier/rust-dapr/tree/master/dapr/examples) folder contains a Dapr enabled app that receives events ([client](https://github.com/flier/rust-dapr/blob/master/dapr/examples/client.rs)), and a caller that invokes the Dapr API ([caller](https://github.com/flier/rust-dapr/blob/master/dapr/examples/caller.rs)).

#### Run the client

```sh
$ cargo +beta build --example client
$ dapr run --log-level debug --protocol grpc --port 3500 --grpc-port 3600 --app-id client --app-port 4000 target/debug/examples/client
```

#### Run the caller

```sh
$ DAPR_GRPC_PORT=3600 cargo +beta run --example caller
```

**Note:** If you don't setup a Dapr binding, expect the error message
> Error: Status { code: Unknown, message: "ERR_INVOKE_OUTPUT_BINDING: couldn\'t find output binding storage" }

## License

This project is licensed under the [MIT](https://spdx.org/licenses/MIT.html) license.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Tonic by you, shall be licensed as MIT, without any additional terms or conditions.
