# swc-plugin-transform-remove-imports

A Rust versions of [babel-plugin-transform-remove-imports](https://github.com/uiwjs/babel-plugin-transform-remove-imports).

Modular import plugin for swc. Also works for cjs to delete imported CSS to avoid compilation errors.

## Installation

**npm:**

```sh
npm i -D swc-plugin-transform-remove-imports
```

**yarn:**

```sh
yarn add -D swc-plugin-transform-remove-imports
```

You can check the compatibility of versions on https://plugins.swc.rs/

## Usage

Via `.swcrc`

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "swc-plugin-transform-remove-imports",
          {
            "test": "\\.(less|css)$"
          }
        ]
      ]
    }
  }
}
```

### Support import

```js
// Input Code
import "./index.less";
import "./index.main.less";
import { Button } from "uiw";
import { Select } from "@uiw/core";

// Output   ↓ ↓ ↓ ↓ ↓ ↓
import { Button } from "uiw";
import { Select } from "@uiw/core";
```

Output Result

```diff
- import './index.less';
- import './index.main.less';
import { Button } from 'uiw';
import { Select } from '@uiw/core';
```

#### Support `require`

## Options

### `test`

Type: `Regex | Regex[]`

A regular expression to match the imports that will be removed.

### `remove`

Optional. Possible values: `'effects'`

Removing only side effects imports.

```js
// Input Code
import "foo";
import Foo from "foo";

// Output Code  ↓ ↓ ↓ ↓ ↓ ↓
import Foo from "foo";
```

## License

[MIT](./LICENSE)
