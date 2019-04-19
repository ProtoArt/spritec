// All the code for interacting with the spritec_preview WASM module.

module.exports = {ready};

// This variable will be populated with the module and instance when it is ready
let wasm = {
  ready: false,
};

let listeners = [];

// Register an event listener for when the module is ready
function ready(listener) {
  listeners.push(listener);
  notify();
}

function notify() {
  if (wasm.ready) {
    for (const listener of listeners) {
      listener(wasm);
    }
    listeners = []
  }
}

const imports = {
  env: {alert},
};

const request = fetch('build/wasm/spritec_preview.wasm');
WebAssembly.instantiateStreaming(request, imports)
  .then(({module, instance}) => wasm = {ready: true, module, instance})
  .then(notify);
