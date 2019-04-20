// A canvas element that represents a rendered 3D model
//
// Automatically resizes to fit its container while also maintaining the aspect
// ratio of the rendered image.
class ModelCanvas {
  constructor({renderer, element, modelPath, imageWidth, imageHeight}) {
    this.renderer = renderer;
    this.el = element;
    this.ctx = this.el.getContext('2d');
    this.modelPath = modelPath;
    // The dimensions to render at, not necessarily the dimensions of the canvas
    this.imageWidth = imageWidth;
    this.imageHeight = imageHeight;
    // The actual dimensions of the canvas (set during resize)
    this.width = null;
    this.height = null;
    // The scale factor to render at (integer)
    this.scale = null;

    this.resize();
  }

  // Determine the size of the canvas and redraw as necessary
  resize() {
    // Want to preserve the aspect ratio of the image while keeping it in bounds
    // https://stackoverflow.com/a/1106367/551904
    const parent = this.el.parentElement;
    const parentWidth = parent.clientWidth;
    const parentHeight = parent.clientHeight;

    // Going to find the scale factor we would have to use to get up to the max
    // width and max height and then scale by the lower of those factors to get
    // within the box.
    const widthScale = parentWidth / this.imageWidth;
    const heightScale = parentHeight / this.imageHeight;

    // The scale factor must be an integer because pixels are indivisible
    const scale = Math.floor(Math.min(widthScale, heightScale));

    this.width = this.imageWidth * scale;
    this.height = this.imageHeight * scale;
    this.scale = scale;

    // Update the canvas size
    this.el.width = this.width;
    this.el.height = this.height;

    // Re-render
    console.log(this.width, this.height, this.scale);
  }

  render() {
    this.ctx.clearRect(0, 0, this.width, this.height);
    this.ctx.putImageData(this.renderer.imageData(), 0, 0);
  }
}

module.exports = ModelCanvas;
