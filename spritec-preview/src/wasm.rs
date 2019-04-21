mod alloc;
mod renderer;
mod console_log;
mod panic_hook;

pub mod str;
pub mod io;

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

    // The panic handler relies on the logging support, so if that doesn't work we're really not
    // in a good state.
    panic_hook::set_once();
}
