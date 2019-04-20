const { copyStringFromWasm } = require('./string.js');

module.exports = {
  console_error(wasmExports, message_ptr) {
    const message = copyStringFromWasm(wasmExports, message_ptr);
    console.error(message);
  },
  console_warn(wasmExports, message_ptr) {
    const message = copyStringFromWasm(wasmExports, message_ptr);
    console.warn(message);
  },
  console_info(wasmExports, message_ptr) {
    const message = copyStringFromWasm(wasmExports, message_ptr);
    console.info(message);
  },
  console_log(wasmExports, message_ptr) {
    const message = copyStringFromWasm(wasmExports, message_ptr);
    console.log(message);
  },
  console_debug(wasmExports, message_ptr) {
    const message = copyStringFromWasm(wasmExports, message_ptr);
    console.debug(message);
  },
};
