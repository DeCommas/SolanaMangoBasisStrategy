#[macro_use]
pub mod error;

pub mod ids;
pub mod instruction;
pub mod matching;
pub mod oracle;
pub mod processor;
pub mod queue;
pub mod state;
pub mod utils;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

#[cfg(feature = "wasm32-compat")]
fn _wasm_usize_check() {
    unsafe {
        std::mem::transmute::<usize, [u8; 4]>(0); // wasm32-compat feature enabled on 64bit target
    }
}

#[cfg(not(feature = "wasm32-compat"))]
fn _wasm_usize_check() {
    unsafe {
        std::mem::transmute::<usize, [u8; 8]>(0); // To build mango for wasm32 enable wasm32-compat feature
    }
}
