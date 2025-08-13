// Example block plugin used in documentation.
// The module exposes a `register` function which receives the base Block
// class and a helper for registration. The plugin can be reloaded at runtime
// with `reloadPlugins(['./my-block.js'])`.
export function register({ Block, registerBlock }) {
  class MyBlock extends Block {
    constructor(id, x, y, w, h, label, color, extras = {}) {
      super(id, x, y, w, h, label, color);
      this.extras = extras;
    }
    draw(ctx) {
      super.draw(ctx);
      ctx.strokeStyle = this.extras.outline || 'red';
      ctx.strokeRect(this.x, this.y, this.w, this.h);
    }
  }

  registerBlock('MyBlock', MyBlock);
}
