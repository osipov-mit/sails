use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    sails_client_gen::generate_client_from_idl(
        Path::new("../wasm/demo.idl"),
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("demo_client.rs"),
    )
    .unwrap();
}