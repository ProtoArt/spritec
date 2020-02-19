/**
 * Base component for other components to extend from
 */
class Component {
  constructor(element, store) {
    this.props = {};
    this.dispatch = store.dispatch;

    this.onCreate(element);
    this._mapStateToProps = this.mapStateToProps();
    if (this._updateComponent(store.getState())) {
      this.render();
    }
  }

  /**
   * Returns a dictionary where the keys will be the keys for this.props[key]
   * and the values are how to select them from the state
   */
  mapStateToProps() { return {}; }

  /**
   * Called when the component constructor is called, before the first render.
   */
  onCreate(_) {}

  /**
   * Called when the component values are updated but before it is rendered
   * again.
   */
  componentDidUpdate(_prevProps) {}

  /**
   * Updates the props of this Component.
   * Returns true if the component should render again because a prop changed.
   */
  _updateComponent(newState) {
    // Get a copy of the props before we update it.
    let prevProps = Object.assign({}, this.props);

    let shouldRender = Object.entries(this._mapStateToProps)
      .reduce((shouldRender, entry) => {
        let prevValue = this.props[entry[0]];
        let newValue = (entry[1])(newState);

        this.props[entry[0]] = newValue;

        return shouldRender || (newValue !== prevValue);
      }, false)

    if (shouldRender) this.componentDidUpdate(prevProps);
    return shouldRender;
  }

  /**
   * Calls with updated values in this.props from the state to render
   */
  render() {}
}

module.exports = Component;
