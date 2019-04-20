const spritec_preview = require('./spritec_preview');
const ModelCanvas = require('./modelcanvas.js');

spritec_preview.then((spritec) => {
  console.log(spritec);
  const renderer = spritec.renderer();
  console.log(renderer);

  const modelPath = '../samples/bigboi/obj/bigboi_rigged_000001.obj';
  const mtlPath = '../samples/bigboi/obj/bigboi_rigged_000001.mtl';

  setupCanvas(renderer);
});

function setupCanvas(renderer) {
  const canvasEl = document.getElementById('app-canvas');
  const modelCanvas = new ModelCanvas({
    renderer,
    element: canvasEl,
    imageWidth: 64,
    imageHeight: 64,
  });

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
      render();
    }
  })
}
