# Open Audio Search frontend

This is the main user interface for Open Audio Search. It's a React single page application.

## Requirements

You need Node.js and npm or yarn. yarn is recommended because it's much faster.

On Debian based systems use the following to install both Node.js and yarn:
```bash
curl -sL https://deb.nodesource.com/setup_12.x | sudo -E bash -
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update
sudo apt install yarn nodejs
```

## Development

For development `webpack-dev-server` is included. In this folder, run `yarn` to install all dependencies and then `yarn start` to start the live-reloading development server. Then open the UI at [http://localhost:4000](http://localhost:4000). In development mode, the UI expects a running oas_core server at `http://localhost:8080`.

## Deployment

Make sure to run `yarn build` in this directory after pulling in changes. The `oas_core` server serves the UI at `/ui` from the `dist/` folder in this directory. 