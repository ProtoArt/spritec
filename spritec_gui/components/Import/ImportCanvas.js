const spritec = require('spritec_binding');
const Component = require('../../lib/Component');

class ImportCanvas extends Component {
  onCreate(canvas) {
    this.state = {
      canvas,
      ctx: canvas.getContext('2d'),
    };
    this.state.ctx.imageSmoothingEnabled = false;

    // TODO: Allow scaling with CSS with `image-rendering: pixelated`, see
    // https://developer.mozilla.org/en-US/docs/Web/API/Element/wheel_event
  }

  mapStateToProps() {
    return {
      width: (state) => state.import.width,
      height: (state) => state.import.height,
      file: (state) => state.import.selectedFile,
    };
  }

  render() {
    const { canvas, ctx } = this.state;
    const { width, height, file } = this.props;

    canvas.width = width;
    canvas.height = height;

    if (file === null) return;

    // TODO: use offscreen canvas when calling spritec
    let imageBuffer = new Uint8ClampedArray(spritec.render_sprite(
      file,
      width,
      height
    ));
    let imageData = new ImageData(imageBuffer, width);

    createImageBitmap(imageData).then((bitmap) => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(bitmap, 0, 0);
      bitmap.close();
    });
  }
}

module.exports = ImportCanvas;
