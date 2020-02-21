const Component = require('../../lib/Component');
const {exportSprites} = require('./actions');

class ExportModal {
  constructor(element, store) {
    element.querySelector('#export-form').onsubmit = () => {
      // Use onsubmit for form validation but don't submit
      event.preventDefault();

      store.dispatch(exportSprites());
    }
  }
}

module.exports = ExportModal;
