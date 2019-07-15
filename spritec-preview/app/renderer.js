const spritec_preview = require('./spritec_preview');
const ModelCanvas = require('./modelcanvas.js');

spritec_preview.then((spritec) => {
  const modelPath = '../samples/bigboi/obj/bigboi_rigged_000001.obj';
  const renderer = spritec.renderer({
    path: modelPath,
    width: 64,
    height: 64,
    scale: 3,
  });

  const modelCanvas = setupCanvas(renderer);
  setupForm(renderer, modelCanvas);
});

function setupCanvas(renderer) {
  const canvasEl = document.getElementById('app-canvas');
  const modelCanvas = new ModelCanvas(renderer, canvasEl);

  window.addEventListener('resize', () => {
    modelCanvas.rescale();
  });

  let rotation = 0.0;
  let dragging = false;
  document.addEventListener('mousedown', (e) => {
    dragging = true;
  });
  document.addEventListener('mouseup', (e) => {
    dragging = false;
  });
  document.addEventListener('mousemove', (e) => {
    if (dragging) {
      const dx = e.movementX / 1000.0;
      rotation += dx * Math.PI;
      renderer.setViewYRotation(rotation);
      modelCanvas.render();
    }
  });

  return modelCanvas;
}

function setupForm(renderer, modelCanvas) {
  const widthEl = document.getElementById('image-width');
  const heightEl = document.getElementById('image-height');

  widthEl.addEventListener('input', (e) => {
    const width = Number(e.target.value);
    // Ignore updates that aren't valid
    if (width > 0) {
      renderer.setWidth(width);
      modelCanvas.rescale();
    }
  });

  heightEl.addEventListener('input', (e) => {
    const height = Number(e.target.value);
    // Ignore updates that aren't valid
    if (height > 0) {
      renderer.setHeight(height);
      modelCanvas.rescale();
    }
  });
}
