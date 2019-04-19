// All the code for interacting with the spritec_preview WASM module.

const imports = {
  env: {alert},
};

const request = fetch('build/wasm/spritec_preview.wasm');
module.exports = WebAssembly.instantiateStreaming(request, imports);
