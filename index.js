// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`
// will work here one day as well!
const rust = import("./pkg");

rust
  .then((m) => {
    var start = window.performance.now();
    let strings = m.js_generate("misunderstanding");
    var end = window.performance.now();
    console.log(`Execution time: ${end - start} ms`);
  })
  .catch(console.error);
