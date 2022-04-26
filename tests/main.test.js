const swc = require('@swc/core');

const javascript = createTransformer();

test('demo', async () => {
  expect(
    await javascript`
      import {createStore} from 'effector';

      const $demo = createStore(0);
    `,
  ).toMatchInlineSnapshot(`
    "import { createStore } from \\"effector\\";
    var $demo = createStore(0);
    "
  `);
});

function createTransformer(options = {}) {
  return async function transform(strings, ...keys) {
    const source = String.raw(strings, ...keys);

    return swc
      .transform(source, {
        filename: 'test.javascript',
        ...options,
      })
      .then((output) => output.code);
  };
}
