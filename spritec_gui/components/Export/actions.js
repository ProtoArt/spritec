const spritec = require('../../spritec');
const {dialog, shell} = require('electron').remote;
const {actions} = require('../Import/slice');

const exportSprites = (element) => async (dispatch, getState) => {
  dispatch(actions.startSubmit());

  try {
    const name = element.querySelector('#export-name').value;
    const scale = Number(element.querySelector('#export-scale').value);
    // spritesheet or frame
    const pngFormat = element.querySelector('input[name="png-format"]:checked').value;
    // gif or png
    const imageFormat = element.querySelector('.uk-tab>.uk-active').textContent;

    const {
      import: {selected: {
        width,
        height,
        camera,
        animation,
        animation_total_steps,
      }},
    } = getState();

    const {canceled, filePath} = await dialog.showSaveDialog({
      defaultPath: `${name}.${imageFormat}`,
    });

    if (!canceled) {
      if (imageFormat === 'gif') {
        spritec.saveGif(
          filePath,
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
        shell.openItem(filePath);
      }
      if (imageFormat === 'png') {
        // TODO: implement png
        alert('not yet implemented');
      }
    }
  } catch (err) {
    console.error(err);
  } finally {
    dispatch(actions.endSubmit());
  }
};

module.exports = {exportSprites};
