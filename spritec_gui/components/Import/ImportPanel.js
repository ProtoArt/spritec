const {actions} = require('./slice');
const Component = require('../../lib/Component');

class ImportPanel extends Component {
  onCreate(element) {
    this.state = {
      inputWidth: element.querySelector('#input-width'),
      inputHeight: element.querySelector('#input-height'),
      inputSteps: element.querySelector('#input-steps'),
    }
    this.state.inputWidth.onchange = (event) => {
      this.dispatch(actions.setWidth(Number(event.target.value)));
    }
    this.state.inputHeight.onchange = (event) => {
      this.dispatch(actions.setHeight(Number(event.target.value)));
    }
    this.state.inputSteps.oninput = (event) => {
      this.dispatch(actions.setAnimationSteps(Number(event.target.value)));
    }
  }

  mapStateToProps() {
    return {
      width: (state) => state.import.selected.width,
      height: (state) => state.import.selected.height,
      steps: (state) => state.import.selected.animation_total_steps,
      animation: (state) => state.import.selected.animation,
    };
  }

  render() {
    this.state.inputWidth.value = this.props.width;
    this.state.inputHeight.value = this.props.height;
    this.state.inputSteps.value = this.props.steps;
    this.state.inputSteps.disabled = (this.props.animation === null);
  }
}

module.exports = ImportPanel;
