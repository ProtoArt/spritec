const { createWasmString, copyStringFromWasm } = require('./string.js');
const Renderer = require('./renderer.js');

// Class to encapsulte all interaction with the spritec_preview wasm module
class Spritec {
  constructor({module, instance}) {
    this.module = module;
    this.instance = instance;
  }

  // Returns the methods exported by the module
  exports() {
    return this.instance.exports;
  }

  // Returns the memory buffer of the module instance
  memoryBuffer() {
    return this.exports().memory.buffer;
  }

  // Create a new renderer for interacting with the web assembly module
  renderer({path, width, height, scale}) {
    return new Renderer(this, {path, width, height, scale});
  }

  // Create a new string in the web assembly memory and return the pointer to it
  fromString(str) {
    return createWasmString(this.exports(), str);
  }

  // Copies a string from WASM and returns a regular JS string
  intoString(str_ptr) {
    return copyStringFromWasm(this.exports(), str_ptr);
  }
}

module.exports = Spritec;
