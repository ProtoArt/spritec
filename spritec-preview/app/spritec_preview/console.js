const { copyStringFromWasm } = require('./string.js');

module.exports = {
  console_error(wasmExports, messagePtr) {
    const message = copyStringFromWasm(wasmExports, messagePtr);
    console.error(message);
  },
  console_warn(wasmExports, messagePtr) {
    const message = copyStringFromWasm(wasmExports, messagePtr);
    console.warn(message);
  },
  console_info(wasmExports, messagePtr) {
    const message = copyStringFromWasm(wasmExports, messagePtr);
    console.info(message);
  },
  console_log(wasmExports, messagePtr) {
    const message = copyStringFromWasm(wasmExports, messagePtr);
    console.log(message);
  },
  console_debug(wasmExports, messagePtr) {
    const message = copyStringFromWasm(wasmExports, messagePtr);
    console.debug(message);
  },
};
