class Renderer {
  constructor(spritec) {
    this.spritec = spritec;
    this.ptr = this.spritec.exports().renderer_new();

    const width = 64;
    const height = 64;
    const scale = 12;
    this.image = {width, height, scale};

    this._fetchImage();
  }

  // Must call destroy before this goes out of scope or else we will leak memory
  // in the web assembly module. The Renderer object *cannot* be used after this
  // method is called.
  destroy() {
    this.spritec.exports().renderer_delete(this.ptr)
  }

  // Returns an ImageData instance suitable for rendering to a canvas
  imageData() {
    return this.image.data;
  }

  // Perform a render and update the image data with the resulting image
  render() {
    const ptr = this.spritec.exports().renderer_render(this.ptr);
    this.ptr = ptr;
    this._fetchImage();
  }

  _fetchImage() {
    const imagePtr = this.spritec.exports().image_data(this.ptr);

    const {width, height, scale} = this.image;
    const buffer = new Uint8ClampedArray(
      this.spritec.memoryBuffer(),
      imagePtr,
      4 * width * scale * height * scale,
    );
    const data = new ImageData(buffer, width * scale, height * scale);

    this.image.ptr = imagePtr;
    this.image.buffer = buffer;
    this.image.data = data;
  }
}

module.exports = Renderer;
