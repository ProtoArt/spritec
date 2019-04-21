class Renderer {
  constructor(spritec, {path, width, height, scale}) {
    this.spritec = spritec;

    const pathPtr = this.spritec.fromString(path);
    this.ptr = this.spritec.exports().renderer_new(pathPtr, width, height, scale);

    // A mapping from each variant of ConfigureRenderer to a number that can be
    // passed to the WASM to set the configuration option
    const configOptsPtr = this.spritec.exports().renderer_config_options();
    this.configOpts = JSON.parse(this.spritec.intoString(configOptsPtr));

    this.image = {width, height, scale};

    this._fetchImage();
  }

  // Must call destroy before this goes out of scope or else we will leak memory
  // in the web assembly module. The Renderer object *cannot* be used after this
  // method is called.
  destroy() {
    this.spritec.exports().renderer_delete(this.ptr)
  }

  // Returns the width of the image rendered by this renderer (without taking
  // scale into account)
  imageWidth() {
    return this.image.width;
  }

  // Returns the height of the image rendered by this renderer (without taking
  // scale into account)
  imageHeight() {
    return this.image.height;
  }

  // Returns an ImageData instance suitable for rendering to a canvas
  imageData() {
    return this.image.data;
  }

  // Sets the scale property of the renderer
  // Does *not* trigger a re-render
  setScale(scale) {
    const opt = this.configOpts.Scale;
    this.spritec.exports().renderer_config_usize(this.ptr, opt, scale);
    this.image.scale = scale;
  }

  // Sets the Y rotation of the renderer
  // Does *not* trigger a re-render
  setViewYRotation(rotation) {
    const opt = this.configOpts.ViewYRotation;
    this.spritec.exports().renderer_config_f32(this.ptr, opt, rotation);
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
