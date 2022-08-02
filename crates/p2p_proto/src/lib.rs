mod starknet {
    include!(concat!(env!("OUT_DIR"), "/starknet.rs"));
}

pub use starknet::*;
