const spritec = require('../../spritec');
const {actions} = require('../Import/slice');

const exportSprites = (element) => (dispatch, getState) => {
  dispatch(actions.startSubmit());
  const name = element.querySelector('#export-name').value;
  const scale = Number(element.querySelector('#export-scale').value);
  // spritesheet or frame
  const pngFormat = element.querySelector('input[name="png-format"]:checked').value;
  // gif or png
  const imageFormat = element.querySelector('.uk-tab>.uk-active').textContent;

  const savePath = 'test.gif';
  const {
    import: { selected: {
      width,
      height,
      camera,
      animation,
      animation_total_steps,
    }},
  } = getState();

  spritec.saveGif(
    savePath,
    width,
    height,
    scale,
    new Float32Array(camera.position).buffer,
    new Float32Array(camera.rotation).buffer,
    new Float32Array(camera.scale).buffer,
    camera.aspect_ratio,
    camera.near_z,
    camera.far_z,
    camera.fov,
    animation,
    animation_total_steps,
  );
  dispatch(actions.endSubmit());
};

module.exports = {exportSprites};
