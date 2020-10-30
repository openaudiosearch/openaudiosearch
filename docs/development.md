# Development setup

Follow everything in [./install.md](install.md)

Create a `.env` file with a suitable config.

### Development mode

The server can be reloaded automatically when application code changes. You can enable it by setting the `oas_dev` env config, or starting the server with `OAS_DEV=1 server.py`.

### Frontend development

```
cd frontend
yarn
yarn start
```

### Inspect the Redis databaes

[`redis-commander`](https://www.npmjs.com/package/redis-commander) is a useful tool to inspect the Redis database. 

```bash
# install redis-commander
yarn global add redis
# or: npm install -g redis

# start redis-commander
redis-commander
```

Now, open your browser at [http://localhost:8081/](http://localhost:8081/)
