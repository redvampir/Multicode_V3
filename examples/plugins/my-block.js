// Example block plugin used in documentation.
// The module exposes a `register` function which receives the base Block
// class and a helper for registration.
export function register({ Block, registerBlock }) {
  class MyBlock extends Block {
    draw(ctx) {
      super.draw(ctx);
      ctx.strokeStyle = 'red';
      ctx.strokeRect(this.x, this.y, this.w, this.h);
    }
  }

  registerBlock('MyBlock', MyBlock);
}
