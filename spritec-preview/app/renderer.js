const { dialog } = require('electron').remote;

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
  const modelFileEl = document.getElementById('model-file');
  const widthEl = document.getElementById('image-width');
  const heightEl = document.getElementById('image-height');
  const saveEl = document.getElementById('save-image');

  modelFileEl.addEventListener('change', (e) => {
    if (!e.target.files.length) {
      // No files selected
      return;
    }

    const path = e.target.files[0].path;
    // Reset the input so the path doesn't remain visible for no reason
    e.target.value = '';

    renderer.load(path);
    modelCanvas.render();
  });

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

  saveEl.addEventListener('click', (e) => {
    const path = dialog.showSaveDialog({
      title: "Save pose as PNG",
    });

    if (!path) {
      return;
    }

    modelCanvas.save(path);
  });
}
