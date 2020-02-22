const Component = require('../../lib/Component');
const {actions} = require ('../Import/slice');
const {exportSprites} = require('./actions');

class ExportModal extends Component {
  onCreate(element) {
    this.state = {
      scaleText: element.querySelector('#export-scale-text'),
      scaleElement: element.querySelector('#export-scale'),
      submitButton: element.querySelector('#export-modal-button'),
    };

    element.querySelector('#export-form').onsubmit = event => {
      // Use onsubmit for form validation but don't actually submit
      event.preventDefault();
      this.dispatch(exportSprites(event.target));
    };

    this.state.scaleElement.oninput = event => {
      this.dispatch(actions.setScale(Number(event.target.value)));
    };
  }

  mapStateToProps() {
    return {
      scale: state => state.import.selected.scale,
      submitting: state => state.import.submitting,
      width: state => state.import.selected.width,
      height: state => state.import.selected.height,
    };
  }

  render() {
    const {width, height, scale, submitting} = this.props;
    this.state.submitButton.disabled = submitting;
    this.state.scaleText.innerText =
      `${scale}x (${width * scale}x${height * scale})`;
    this.state.scaleElement.value = scale;
  }
}

module.exports = ExportModal;
