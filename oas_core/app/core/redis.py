from redis import StrictRedis as Redis, ConnectionPool
from app.config import config

redis = Redis.from_url(config.redis_url)
