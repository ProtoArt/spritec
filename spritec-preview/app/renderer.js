const spritec = require("spritec-preview");

spritec.initialize();

const canvasEl = document.getElementById('canvas');
const ctx = canvasEl.getContext('2d');
const arr = new Uint8ClampedArray(40000);

// Iterate through every pixel
for (let i = 0; i < arr.length; i += 4) {
  arr[i + 0] = 0;    // R value
  arr[i + 1] = 190;  // G value
  arr[i + 2] = 0;    // B value
  arr[i + 3] = 255;  // A value
}

// Initialize a new ImageData object
let imageData = new ImageData(arr, 200);

// Draw image data to the canvas
ctx.putImageData(imageData, 20, 20);

setTimeout(() => {
  for (let i = 0; i < arr.length; i += 4) {
    arr[i + 0] = 190;  // R value
    arr[i + 1] = 0;    // G value
    arr[i + 2] = 0;    // B value
    arr[i + 3] = 255;  // A value
  }
  ctx.putImageData(imageData, 20, 20);
}, 1000);
