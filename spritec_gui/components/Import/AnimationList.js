const Component = require('../../lib/Component');
const {actions: {setAnimation, stopAnimation}} = require('./slice');

const NO_ANIMATION_VALUE = '@@spritec/NO_ANIMATION';

class AnimationList extends Component {
  onCreate(element) {
    this.state = {
      listElement: element,
    }
    this.state.listElement.onchange = (event) => {
      if (event.target.value === NO_ANIMATION_VALUE) {
        this.dispatch(stopAnimation());
      } else {
        this.dispatch(setAnimation(
          this.props.animations[Number(event.target.value)]
        ));
      }
    }
  }

  mapStateToProps() {
    return {
      animations: state => state.import.selected.animations,
      selectedAnimation: state => state.import.selected.animation,
    };
  }

  render() {
    const {selectedAnimation, animations} = this.props;
    const {listElement} = this.state;

    // clear the list
    listElement.innerHTML = '';

    listElement.disabled = (animations.length == 0);

    const playGroup = document.createElement('optgroup');
    playGroup.label = 'Play';
    animations.forEach(({name}, index) => playGroup.appendChild(new Option(
      name, // text
      index, // value
      false,  // default selected
      selectedAnimation && selectedAnimation.name === name // selected
    )));

    const stopGroup = document.createElement('optgroup');
    stopGroup.label = 'Stop';
    stopGroup.appendChild(new Option(
      'No Animation',
      NO_ANIMATION_VALUE,
      false,
      selectedAnimation === null,
    ));

    listElement.add(playGroup);
    listElement.add(stopGroup);
  }
}

module.exports = AnimationList;
