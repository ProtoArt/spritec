const spritec = require('spritec_binding');
const Component = require('../../lib/Component');

class ImportCanvas extends Component {
  onCreate(container) {
    const canvas = container.querySelector('#spritec-canvas');
    this.state = {
      canvas,
      ctx: canvas.getContext('2d'),
    };
    this.state.ctx.imageSmoothingEnabled = false;

    canvas.style.transform = `scale(8)`;

    let scale = 8;
    container.onwheel = (event) => {
      event.preventDefault();
      scale += event.deltaY * -0.01;
      scale = Math.max(0.125, scale);
      this.state.canvas.style.transform = `scale(${scale})`;
    }
  }

  mapStateToProps() {
    return {
      width: (state) => state.import.width,
      height: (state) => state.import.height,
      file: (state) => state.import.selectedFile,
    };
  }

  render() {
    const {canvas, ctx} = this.state;
    const {width, height, file} = this.props;

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
