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
    const imagePtr = this.spritec.exports().context_image_data(this.ptr);
    console.log(imagePtr);
    const buffer = new Uint8ClampedArray(
      this.spritec.memoryBuffer(),
      imagePtr,
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

  // Perform a render and update the image data with the resulting image
  render() {
    const ptr = this.spritec.exports().context_render(this.ptr);
    const imagePtr = this.spritec.exports().context_image_data(ptr);
    const width = 64;
    const height = 64;
    const scale = 1;
    const buffer = new Uint8ClampedArray(
      this.spritec.memoryBuffer(),
      imagePtr,
      4 * width * scale * height * scale,
    );
    const data = new ImageData(buffer, width, height);
    this.image.buffer = buffer;
    this.image.data = data;
  }
}

let wasmExports = {};

/**
 * Copy C string from WASM into a JavaScript string
 *
 * C strings are NULL terminated, so this function will continue to read memory
 * until it reaches the NULL terminator '\0'. If no NULL terminator is found,
 * an exception will be thrown.
 */
function copyStringFromWasm(wasmExports, ptr) {
  // Code adapted from: https://github.com/WasmBlock/WasmBlock/blob/bc5959dd7b0d0d5f5ed4033b149591574bad68b6/wasmblock.js#L31

  const orig_ptr = ptr;
  const collectCString = function* () {
    const memory = new Uint8Array(wasmExports.memory.buffer);
    while (memory[ptr] !== 0) {
      if (memory[ptr] === undefined) {
        throw new Error("Tried to read undef mem");
      }
      yield memory[ptr];
      ptr += 1;
    }
  }

  const buffer_as_u8 = new Uint8Array(collectCString())
  const utf8Decoder = new TextDecoder("UTF-8");
  const buffer_as_utf8 = utf8Decoder.decode(buffer_as_u8);
  // Free the allocated string memory once we are done with it
  wasmExports.dealloc_str(orig_ptr);
  return buffer_as_utf8;
}

const imports = {
  env: {
    console_log(message_ptr) {
      const message = copyStringFromWasm(wasmExports, message_ptr);
      console.log(message);
    },
  },
};

const request = fetch('build/wasm/spritec_preview.wasm');
module.exports = WebAssembly.instantiateStreaming(request, imports)
  .then((wasm) => {
    wasmExports = wasm.instance.exports;
    return new Spritec(wasm);
  });
