# Effector SWC Plugin

Plugin for SWC which can be used to automatically change imports (scope/no scope), insert sids (for scope) or for debugging.

## ⚠️ Please notice, that SWC Plugin API itself is not stable yet ⚠️

This means that `@effector/swc-plugin` **can and probably will be affected by breaking changes in the SWC Plugins API** in the future.
At the moment this plugins is seems to be stable enough. But if you need guaranteed stability - prefer `effector/babel-plugin` instead.

When SWC Plugin API will be stabilized, we will be able to declare `@effector/swc-plugin` as stable too.

# Installation

You can use any package manager that you want.
You also need to make sure that `@swc/core` is installed, no matter where are you calling the transforms from. Be it `unplugin-swc` or `@swc/cli`.

```bash
pnpm add -D @effector/swc-plugin @swc/core
```

*or*

```bash
npm add -D @effector/swc-plugin @swc/core
```

*or*

```bash
yarn add -D @effector/swc-plugin @swc/core
```

# Usage
In the simplest case, it can be used without any configuration.

`.swcrc`
```json
{
  "$schema": "https://json.schemastore.org/swcrc",
  "jsc": {
    "experimental": {
      "plugins": ["@effector/swc-plugin"]
    }
  }
}
```

## Configuration

### importName
- Type: `string | string[]`
- Default: `['effector', 'effector/compat']`

Specify import name or names to process by plugin.
Import should be used in the code as specifed.

### factories
- Type: `string[]`

Accepts an array of module names which exports treat as custom factories therefore each function call provides unique prefix for sids of units inside them. Used to SSR(Server Side Rendering) and it's not required for client-only application (except if you want to test your app).

- Factories can have any amount of arguments.
- Factories can create any amount of units.
- Factories can call any effector methods.
- Factories can call another factories from others modules.
- Modules with factories can export any amount of functions.
- Factories should be compiled with `effector/babel-plugin` or `@effector/swc-plugin` as well as code which use them.

`.swcrc`
```json
{
  "$schema": "https://json.schemastore.org/swcrc",
  "jsc": {
    "experimental": {
      "plugins": [
        "@effector/swc-plugin",
        {
          "factories": ["src/createEffectStatus", "~/createCommonPending"]
        }
      ]
    }
  }
}
```

`./src/createEffectStatus.js`
```js
import {rootDomain} from './rootDomain'

export function createEffectStatus(fx) {
  const $status = rootDomain
    .createStore('init')
    .on(fx.finally, (_, {status}) => status)
    
  return $status
}
```

`./src/statuses.js`
```js
import {createEffectStatus} from './createEffectStatus'
import {fetchUserFx, fetchFriendsFx} from './api'

export const $fetchUserStatus = createEffectStatus(fetchUserFx)
export const $fetchFriendsStatus = createEffectStatus(fetchFriendsFx)
```

Import `createEffectStatus` from `./createEffectStatus` was treated as factory function so each store created by it has its own sid and will be handled by serialize independently, although without `factories` they will share the same `sid`.

### bindings
- Type: `{react?: {scopeReplace?: bool}, solid?: {scopeReplace?: bool}} | undefined`

If `scopeReplace` is enabled for the view library, imports will be replaced from `effector-{viewLib}` to `effector-{viewLib}/scope`.
This config might get additional fields (nested as well) later.

### addNames
- Type: `boolean`
- Default: `true`

Add names to units factories calls. Useful for minification and obfuscation of production builds.

### addLoc
- Type: `boolean`
- Default: `false`

Add location to methods' calls. Used by devtools, for example effector-logger.

### debugSids
- Type: `boolean`
- Default: `false`

Add path of a file and a variable name whether a unit was defined to a sid. Useful for debugging SSR.

## Bundlers
Vite + Solid (SSR)

To use vite + solidjs you have to do the following:

1. Install dependencies
   - ```bash
     pnpm add -D vite vite-plugin-solid solid-js 
     pnpm add -D unplugin-swc 
     pnpm add -D effector effector-solid @effector/swc-plugin
     ```
2. `vite.config.ts` should look like this:
   - ```ts
     // vite.config.ts
     import { defineConfig } from 'vite';
     import solidPlugin from 'vite-plugin-solid';
     import swc from 'unplugin-swc';
     
     export default defineConfig({
        plugins: [
            solidPlugin(),
            swc.vite({
                jsc: {
                    experimental: {
                        plugins: ["@effector/swc-plugin"]
                    }
                }
            }),
        ],
     });
     ```
     
Or you can store `jsc` field in `.swcrc` instead.
