const path = require('path');

// In Nix environments, we want to use
// a custom NODE_PATH defined in shell.nix
// Otherwise, we want to omit this config entirely.
const resolve = process.env.NODE_PATH
              ? {
                  resolve: {
        modules: [process.env.NODE_PATH],
      },
    }
  : {};

module.exports = (env) => ({
  ...resolve,
  entry: {
    tezos_js_bridge: "./src/tezos_js_bridge.js",
  },
  output: {
      filename: "[name].bundle.js",
      path: path.resolve(__dirname, 'scripts'),
  },
    mode: env.dev ? "development" : "production",
    target: "node",
});
