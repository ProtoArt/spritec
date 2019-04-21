module.exports = {
  createWasmString,
  copyStringFromWasm,
};

// Creates a new NULL-terminated string in the WebAssembly memory for the
// given string. Space is allocated for the length of the given string + 1
// and then the entire string is encoded and stored as UTF-8 with a
// NULL-terminator at the end.
function createWasmString(wasmExports, str) {
  // Code adapted from: https://github.com/WasmBlock/WasmBlock/blob/bc5959dd7b0d0d5f5ed4033b149591574bad68b6/wasmblock.js#L71

  const utf8Encoder = new TextEncoder("UTF-8");
  const string_buffer = utf8Encoder.encode(str);
  const len = string_buffer.length;
  // +1 because of NULL-terminator
  const ptr = wasmExports.alloc(len+1);

  const memory = new Uint8Array(wasmExports.memory.buffer, ptr, len+1);
  memory.set(string_buffer);
  memory[ptr+len] = 0;

  return ptr;
}

// Copy C string from WASM into a JavaScript string
//
// C strings are NULL terminated, so this function will continue to read memory
// until it reaches the NULL terminator '\0'. If no NULL terminator is found,
// an exception will be thrown.
function copyStringFromWasm(wasmExports, ptr) {
  // Code adapted from: https://github.com/WasmBlock/WasmBlock/blob/bc5959dd7b0d0d5f5ed4033b149591574bad68b6/wasmblock.js#L31

  let str_ptr = ptr;
  const collectCString = function* () {
    const memory = new Uint8Array(wasmExports.memory.buffer);
    while (memory[str_ptr] !== 0) {
      if (memory[str_ptr] === undefined) {
        throw new Error("Tried to read undefined memory");
      }
      yield memory[str_ptr];
      str_ptr += 1;
    }
  }

  const buffer_as_u8 = new Uint8Array(collectCString())
  const utf8Decoder = new TextDecoder("UTF-8");
  const buffer_as_utf8 = utf8Decoder.decode(buffer_as_u8);
  // Free the allocated string memory once we are done with it
  wasmExports.dealloc_str(ptr);
  return buffer_as_utf8;
}
