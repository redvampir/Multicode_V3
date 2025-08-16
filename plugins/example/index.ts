export function register({ Block, registerBlock }: any) {
  class ExampleBlock extends Block {
    constructor(id: string, x: number, y: number) {
      super(id, x, y, 120, 50, 'Example');
    }
  }
  registerBlock('ExampleBlock', ExampleBlock);
}
