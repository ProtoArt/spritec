// All the code for interacting with the spritec_preview WASM module.

const Spritec = require('./spritec.js')
const wasm_io = require('./io.js');

let wasm = {exports: {}};

const applyWasm = (func) => {
  return (...args) => func(wasm.exports, ...args);
};

const imports = {
  env: {
    console_error: applyWasm(wasm_io.console_error),
    console_warn: applyWasm(wasm_io.console_warn),
    console_info: applyWasm(wasm_io.console_info),
    console_log: applyWasm(wasm_io.console_log),
    console_debug: applyWasm(wasm_io.console_debug),
  },
};

const request = fetch('build/wasm/spritec_preview.wasm');
module.exports = WebAssembly.instantiateStreaming(request, imports)
  .then((loaded) => {
    wasm.exports = loaded.instance.exports;
    // Must be called once and only once
    wasm.exports.initialize(process.env.NODE_ENV !== 'production');
    return new Spritec(loaded);
  });
