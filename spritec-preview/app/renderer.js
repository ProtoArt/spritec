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

  setupCanvas(renderer);
});

function setupCanvas(renderer) {
  const canvasEl = document.getElementById('app-canvas');
  const modelCanvas = new ModelCanvas(renderer, canvasEl);

  window.addEventListener('resize', () => {
    modelCanvas.resize();
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
  })
}
