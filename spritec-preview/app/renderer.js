const spritec_preview = require('./spritec_preview.js');

spritec_preview.then((spritec) => {
  console.log(spritec);
  const context = spritec.context();
  console.log(context);

  const canvasEl = document.getElementById('canvas');
  canvasEl.width = 1000;
  canvasEl.height = 820;
  const ctx = canvasEl.getContext('2d');

  let rotation = 0.0;
  const render = () => {
    context.render(rotation);

    ctx.fillStyle = 'red';
    ctx.fillRect(0, 0, 1000, 1000);
    ctx.putImageData(context.imageData(), 20, 20);
  };

  render();

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
});
