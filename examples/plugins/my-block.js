// Пример плагина блока, используемый в документации.
// Модуль предоставляет функцию `register`, которая получает базовый класс Block
// и помощник для регистрации. Плагин можно перезагружать во время работы через
// `reloadPlugins(['./my-block.js'])`.
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
