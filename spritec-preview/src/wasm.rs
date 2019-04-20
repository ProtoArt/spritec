mod alloc;
mod context;
mod console_log;

/// Must be called exactly once when the WASM module is loaded
#[no_mangle]
extern fn initialize(debug: bool) {
    if debug {
        console_log::init_with_level(log::Level::Trace)
            .expect("error initializing logger");
    } else {
        console_log::init()
            .expect("error initializing logger");
    }
}
