// All the code for interacting with the spritec_preview WASM module.

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

  // Create a new context for interacting with the web assembly module
  context() {
    return new Context(this);
  }
}

class Context {
  constructor(spritec) {
    this.spritec = spritec;
    this.ptr = this.spritec.exports().context_new();

    const width = 64;
    const height = 64;
    const scale = 1;
    const buffer = new Uint8ClampedArray(
      this.spritec.memoryBuffer(),
      this.spritec.exports().context_image_data(this.ptr),
      4 * width * scale * height * scale,
    );
    const data = new ImageData(buffer, width, height);
    this.image = {width, height, scale, buffer, data};
  }

  // Must call destroy before this goes out of scope or else we will leak memory
  // in the web assembly module. The Context object *cannot* be used after this
  // method is called.
  destroy() {
    this.spritec.exports().context_delete(this.ptr)
  }

  // Returns an ImageData instance suitable for rendering to a canvas
  imageData() {
    return this.image.data;
  }
}

const imports = {
  env: {},
};

const request = fetch('build/wasm/spritec_preview.wasm');
module.exports = WebAssembly.instantiateStreaming(request, imports)
  .then((wasm) => new Spritec(wasm));
