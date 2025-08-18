# Плагины визуализации

Фронтенд поддерживает подключение плагинов, которые могут регистрировать
дополнительные визуальные блоки или иной функционал.  Каждый плагин должен
реализовать интерфейс `VizPlugin` с методом `register`.

```ts
export interface VizPlugin {
  register(registry: PluginRegistry): void;
}
```

Метод `register` получает объект `registry` с функциями, которые плагин
может вызывать для добавления своих компонентов.  В частности, для блоков
доступна функция `registerBlock` и базовый класс `Block`.

## Загрузчик

Файл `frontend/src/plugins/plugin-loader.ts` автоматически ищет модули
`index.ts` или `index.js` в каталоге `plugins/` и вызывает их метод
`register`.

```ts
import { loadPlugins } from '../frontend/src/plugins/plugin-loader';
loadPlugins({ Block, registerBlock });
```

## Пример плагина

В репозитории присутствует пример плагина в каталоге
`plugins/example/index.ts`:

```ts
export function register({ Block, registerBlock }: any) {
  class ExampleBlock extends Block {
    constructor(id: string, x: number, y: number) {
      super(id, x, y, 120, 50, 'Example');
    }
  }
  registerBlock('ExampleBlock', ExampleBlock);
}
```

Плагин регистрирует новый блок `ExampleBlock`, который сразу можно
использовать после загрузки.

Дополнительные сведения о модулях см. в [modules.md](modules.md). Общие возможности описаны в [features.md](features.md), термины — в [glossary.md](glossary.md).
