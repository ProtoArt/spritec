const fs = require('fs');

const { copyStringFromWasm } = require('./string.js');

module.exports = {
  // Opens a file synchronously and loads it into WASM memory.
  // Stores the size of the file in the 32-bit pointer `lengthPtr`.
  // Returns the pointer to the data written to WASM memory.
  readFile(wasmExports, pathStrPtr, lengthPtr) {
    const path = copyStringFromWasm(wasmExports, pathStrPtr);
    const data = fs.readFileSync(path);

    const ptr = wasmExports.alloc(data.length);
    // Create a view into the WASM memory compatible with the Buffer API
    const memoryBuffer = Buffer.from(wasmExports.memory.buffer)
    const copied = data.copy(memoryBuffer, ptr, 0, data.length);
    if (copied !== data.length) {
      throw new Error('Unable to copy entire buffer');
    }

    // Write the length to signify that this was a success
    memoryBuffer.writeUInt32BE(data.length, lengthPtr);

    return ptr;
  },
};
