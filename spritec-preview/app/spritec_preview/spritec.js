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
  renderer() {
    return new Renderer(this);
  }
}

module.exports = Spritec;
