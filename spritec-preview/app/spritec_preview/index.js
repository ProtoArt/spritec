// All the code for interacting with the spritec_preview WASM module.

const Spritec = require('./spritec.js')
const { console_log } = require('./io.js');

let wasm = {exports: {}};

const applyWasm = (func) => {
  return (...args) => func(wasm.exports, ...args);
};

const imports = {
  env: {
    console_log: applyWasm(console_log),
  },
};

const request = fetch('build/wasm/spritec_preview.wasm');
module.exports = WebAssembly.instantiateStreaming(request, imports)
  .then((loaded) => {
    wasm.exports = loaded.instance.exports;
    return new Spritec(loaded);
  });
