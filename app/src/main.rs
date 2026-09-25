//! OSCC desktop entry point — thin binary over the library (the lib stays
//! wasm-testable).

fn main() {
    oscc_app::run();
}
