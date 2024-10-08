import { h } from '@logicflow/core';

import RectNode from '../basic/RectNode';

// 五角星
class StarModel extends RectNode.model {
  override initNodeData(data: any) {
    super.initNodeData(data);
    this.width = 80;
    this.height = 80;
  }
}

class StarView extends RectNode.view {
  override getResizeShape() {
    const { height, width, x, y } = this.props.model;
    const style = this.props.model.getNodeStyle();
    const svgAttr = {
      height,
      width,
      x: x - (1 / 2) * width,
      y: y - (1 / 2) * height,
    };
    const pathAAttrs = {
      ...style,
      d: 'm0.36922,13.46587l12.98695,0l4.01307,-13.36885l4.01307,13.36885l12.98694,0l-10.50664,8.26231l4.01327,13.36885l-10.50665,-8.26253l-10.50664,8.26253l4.01327,-13.36885l-10.50665,-8.26231l0,0z',
    };

    return h('svg', { ...svgAttr, viewBox: '0 0 37 37' }, [
      h('path', {
        ...pathAAttrs,
      }),
    ]);
  }
}

export default {
  model: StarModel,
  type: 'star',
  view: StarView,
};