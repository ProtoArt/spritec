module.exports = {
  /**
   * Convert hex ('#ffffff') into [1, 1, 1]
   */
  hex_to_rgb: hex => {
    // slice(1) to get rid of the '#'
    const all = parseInt(hex.slice(1), 16);
    const r = (all >> 16) & 255;
    const g = (all >> 8) & 255;
    const b = all & 255;

    return new Float32Array([r/255.0, g/255.0, b/255.0]);
  }
};

