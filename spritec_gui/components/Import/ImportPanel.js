const { actions } = require('./slice');
const Component = require('../../lib/Component');

class ImportPanel extends Component {
  onCreate(element) {
    this.state = {
      inputWidth: element.querySelector('#input-width'),
      inputHeight: element.querySelector('#input-height'),
    }
    this.state.inputWidth.onchange = (event) => {
      this.dispatch(actions.setWidth(Number(event.target.value)));
    }
    this.state.inputHeight.onchange = (event) => {
      this.dispatch(actions.setHeight(Number(event.target.value)));
    }
  }

  mapStateToProps() {
    return {
      width: (state) => state.import.width,
      height: (state) => state.import.height,
    };
  }

  render() {
    this.state.inputWidth.value = this.props.width;
    this.state.inputHeight.value = this.props.height;
  }
}

module.exports = ImportPanel;
