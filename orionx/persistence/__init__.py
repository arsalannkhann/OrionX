"""OrionX Persistence Module - Execution state storage."""

from .base import ExecutionStore
from .memory import InMemoryExecutionStore
from .sqlite import SQLiteExecutionStore

__all__ = [
    "ExecutionStore",
    "InMemoryExecutionStore",
    "SQLiteExecutionStore",
]


def get_execution_store() -> ExecutionStore:
    """
    Get the configured execution store based on environment.
    
    Returns the appropriate store based on ORIONX_PERSISTENCE_BACKEND:
    - memory: In-memory (default for testing)
    - sqlite: SQLite file-based (default for production)
    - redis: Redis (for distributed deployments)
    """
    from ..config import get_config, PersistenceBackend
    
    config = get_config()
    
    if config.persistence.backend == PersistenceBackend.MEMORY:
        return InMemoryExecutionStore()
    elif config.persistence.backend == PersistenceBackend.SQLITE:
        return SQLiteExecutionStore(config.persistence.sqlite_path)
    elif config.persistence.backend == PersistenceBackend.REDIS:
        from .redis import RedisExecutionStore
        return RedisExecutionStore(
            url=config.persistence.redis_url,
            prefix=config.persistence.redis_prefix,
            ttl=config.persistence.redis_ttl_seconds,
        )
    else:
        return InMemoryExecutionStore()
