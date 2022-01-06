// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`

const { LibManifestPlugin } = require('webpack');

// will work here one day as well!
const rust = import('./pkg');

rust
    .then(m => {
        let output = m.compile("async fn main() { print(\"Hello, web\").await }");
        console.log(output);
    })
    .catch(console.error);
