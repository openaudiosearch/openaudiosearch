# Open Audio Search frontend

This is the main user interface for Open Audio Search. It's a React single page application.

## Development

For development `webpack-dev-server` is included. In this folder, run `yarn` to install all dependencies and then `yarn start` to start the live-reloading development server. Then open the UI at [http://localhost:4000](http://localhost:4000). In development mode, the UI expects a running oas_core server at `http://localhost:8080`.

## Deployment

Make sure to run `yarn build` in this directory after pulling in changes. The `oas_core` server serves the UI at `/ui` from the `dist/` folder in this directory. 