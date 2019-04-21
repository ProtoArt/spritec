// A canvas element that represents a rendered 3D model
//
// Automatically resizes to fit its container while also maintaining the aspect
// ratio of the rendered image.
class ModelCanvas {
  constructor(renderer, element) {
    this.renderer = renderer;
    this.el = element;
    this.ctx = this.el.getContext('2d');
    // The actual dimensions of the canvas (set during resize)
    this.width = null;
    this.height = null;

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
    const widthScale = parentWidth / this.renderer.imageWidth();
    const heightScale = parentHeight / this.renderer.imageHeight();

    // The scale factor must be an integer because pixels are indivisible
    let scale = Math.floor(Math.min(widthScale, heightScale));
    // Scale must not be <= 0
    scale = scale > 0 ? scale : 1;

    this.width = this.renderer.imageWidth() * scale;
    this.height = this.renderer.imageHeight() * scale;

    // Update the canvas size
    this.el.width = this.width;
    this.el.height = this.height;

    // Re-render
    this.renderer.setScale(scale);
    this.render();
  }

  render() {
    this.renderer.render();

    this.ctx.clearRect(0, 0, this.width, this.height);
    this.ctx.putImageData(this.renderer.imageData(), 0, 0);
  }
}

module.exports = ModelCanvas;
