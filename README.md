# Open Audio Search

- [Installation](./docs/install.md)
- [Development setup](./docs/development.md)

## Configuration

OAS is configured through an `.env` file in the directory from where you invoke it. To customize the configuration, copy [`.env.default`](`oas_core/.env.default`) in the `oas_core` folder to `.env` and adjust the values.

By default, all data is stored in `/tmp/oas`. To keep models, downloads and intermediate results change the `STORAGE_PATH` setting to a non-temporary path.
