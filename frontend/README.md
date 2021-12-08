# Open Audio Search frontend

This is the default user interface for Open Audio Search. It is a [React](https://reactjs.org/) single page application.

## Requirements

You need Node.js and npm or yarn. yarn is recommended because it's much faster.

On Debian based systems use the following to install both Node.js and yarn:
```bash
curl -sL https://deb.nodesource.com/setup_14.x | sudo -E bash -
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update
sudo apt install yarn nodejs
```

## Development

Run `yarn dev` to start the included [Vite](https://vitejs.dev/) development server. Then open [`http://localhost:4000`](http://localhost:4000) in a web browser.

By default, the API path `/api` is proxied onto `http://localhost:8080`. To change the API URL of the frontend proxy, set the `OAS_URL` environment variable to an OAS API endpoint.

## Packaging

When building the OAS core in release mode, a [`build.rs`](../rust/oas-core/build.rs) script runs `yarn build` in this folder to build the UI with vite. The resulting files (in `dist/`) are bundled into the OAS binary.
