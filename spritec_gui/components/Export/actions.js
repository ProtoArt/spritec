const fs = require('fs');
const path = require('path');
const {dialog, shell} = require('electron').remote;
const spritec = require('../../spritec_binding');
const {actions} = require('../Import/slice');
const {hex_to_rgb} = require('../../lib/color');

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
        light_rotation,
        light_color,
        light_intensity,
      }},
    } = getState();

    const exportObj = await prepareExport(name, imageFormat, pngFormat);

    if (exportObj) {
      const {filePath, saveFn, openFn} = exportObj;
      saveFn(
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
        (animation ? animation_total_steps : 1),
        hex_to_rgb(light_color).buffer,
        light_intensity,
        new Float32Array(light_rotation).buffer,
      );
      openFn(filePath);
    }
  } finally {
    dispatch(actions.endSubmit());
  }
};

/**
 * Returns
 * filePath: Location to save to (may be a file or folder),
 * saveFn: Function to call to save to filePath,
 * openFn: Function to call to open preview of the saved file
 */
const prepareExport = async (name, imageFormat, pngFormat) => {
  if (imageFormat === 'gif') {
    const {canceled, filePath} = await dialog.showSaveDialog({
      defaultPath: `${name}.${imageFormat}`
    });
    if (canceled) return null;
    return {
      filePath,
      saveFn: spritec.saveGif.bind(spritec), 
        openFn: shell.openItem,
    };
  }

  if (imageFormat === 'png' && pngFormat === 'spritesheet') {
    const {canceled, filePath} = await dialog.showSaveDialog({
      defaultPath: `${name}.${imageFormat}`
    });
    if (canceled) return null;
    return {
      filePath,
      saveFn: spritec.saveSpritesheet.bind(spritec),
      openFn: shell.openItem,
    };
  }

  if (imageFormat === 'png' && pngFormat === 'frame') {
    const {canceled, filePaths} = await dialog.showOpenDialog({
      properties: ['openDirectory'],
      multiSelections: false,
    });
    if (canceled || !filePaths) return null;
    const folderPath = path.join(filePaths[0], name);
    return {
      filePath: folderPath,
      saveFn: spritec.saveSprites.bind(spritec),
      openFn: shell.showItemInFolder,
    };
  }

  throw new Error(`${imageFormat} - ${pngFormat} not supported`);
}

module.exports = {exportSprites};
