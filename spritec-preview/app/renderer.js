const spritec_preview = require('./spritec_preview.js');

spritec_preview.then((spritec) => {
  console.log(spritec);
  const context = spritec.context();
  console.log(context);

  const canvasEl = document.getElementById('canvas');
  const ctx = canvasEl.getContext('2d');
  ctx.fillStyle = 'red';
  ctx.fillRect(0, 0, 1000, 1000);
  ctx.putImageData(context.imageData(), 20, 20);
});
