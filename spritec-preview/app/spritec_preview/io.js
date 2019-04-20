const { copyStringFromWasm } = require('./string.js');

module.exports = {
  console_log,
};

function console_log(wasmExports, message_ptr) {
  const message = copyStringFromWasm(wasmExports, message_ptr);
  console.log(message);
}
