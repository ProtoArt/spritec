const spritec = require('../../spritec_binding');
const Component = require('../../lib/Component');

class ImportCanvas extends Component {
  onCreate(container) {
    const canvas = container.querySelector('#spritec-canvas');
    this.state = {
      canvas,
      ctx: canvas.getContext('2d'),
      startTime: performance.now(),
      step: -1,
      renderCanvas: this.renderCanvas.bind(this),
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

    this.renderCanvas = this.renderCanvas.bind(this);
    requestAnimationFrame(this.renderCanvas);
  }

  mapStateToProps() {
    return {
      width: (state) => state.import.selected.width,
      height: (state) => state.import.selected.height,
      path: (state) => state.import.selected.path,
      camera: (state) => state.import.selected.camera,
      animation: (state) => state.import.selected.animation,
      animation_total_steps: (state) => state.import.selected.animation_total_steps,
    };
  }

  componentDidUpdate(prevProps) {
    if (prevProps.path !== this.props.path && this.props.path) {
      spritec.setFile(this.props.path);
    }

    // Only set canvas dimensions when we need to as this call resets the canvas
    const {width, height} = this.props;
    if (prevProps.width !== width || prevProps.height !== height) {
      this.state.canvas.width = width;
      this.state.canvas.height = height;
    }
  }

  async renderCanvas(timestamp) {
    const {canvas, ctx, startTime, step} = this.state;
    const {
      width,
      height,
      camera,
      animation,
      animation_total_steps
    } = this.props;

    const time = (timestamp - startTime) / 1000;
    /**
     * If animation is null then the step is 0. If there is animation, then
     * this produces a graph that cycles through the steps given the time:
     * For example with animation_total_steps = 4:
     *
     * s |      **      **
     * t |    **      **
     * e |  **      **
     * p |**      **
     * --+----------------->
     *    time
     */
    const current_step = animation === null ? 0 : Math.floor(
      (((animation_total_steps - 1) * time) / animation.duration) %
      animation_total_steps
    );

    // Only render if there is something new to render
    if (this.props.path !== null && step !== current_step) {
      this.state.step = current_step;

      let imageBuffer = new Uint8ClampedArray(spritec.render(
        width,
        height,
        new Float32Array(camera.position).buffer,
        new Float32Array(camera.rotation).buffer,
        new Float32Array(camera.scale).buffer,
        camera.aspect_ratio,
        camera.near_z,
        camera.far_z,
        camera.fov,
        animation && animation.name,
        animation_total_steps,
        current_step
      ));

      let imageData = new ImageData(imageBuffer, width);

      const bitmap = await createImageBitmap(imageData);
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(bitmap, 0, 0);
      bitmap.close();
    }

    requestAnimationFrame(this.renderCanvas);
  }
}

module.exports = ImportCanvas;
