"""
OrionX Configuration Module

Centralized configuration from environment variables.
Addresses audit finding: "Hardcoded ExecutionBudget limits"
"""

import os
from dataclasses import dataclass
from typing import Optional
from enum import Enum


class PersistenceBackend(str, Enum):
    """Supported persistence backends."""
    MEMORY = "memory"
    SQLITE = "sqlite"
    REDIS = "redis"


@dataclass
class ExecutionLimits:
    """Configurable execution limits (was hardcoded in ExecutionBudget)."""
    max_db_queries: int = 100
    max_api_calls: int = 10
    max_emails: int = 10
    max_steps: int = 100
    workflow_timeout_seconds: float = 300.0
    step_timeout_seconds: float = 30.0


@dataclass
class PersistenceConfig:
    """Persistence layer configuration."""
    backend: PersistenceBackend = PersistenceBackend.SQLITE
    sqlite_path: str = "./orionx_executions.db"
    redis_url: Optional[str] = None
    redis_prefix: str = "orionx:"
    redis_ttl_seconds: int = 86400 * 7  # 7 days


@dataclass
class OrionXConfig:
    """Main configuration container."""
    limits: ExecutionLimits
    persistence: PersistenceConfig
    debug: bool = False
    log_level: str = "INFO"


def load_config() -> OrionXConfig:
    """
    Load configuration from environment variables.
    
    Environment Variables:
        ORIONX_MAX_DB_QUERIES: Max database queries per workflow (default: 100)
        ORIONX_MAX_API_CALLS: Max external API calls per workflow (default: 10)
        ORIONX_MAX_EMAILS: Max emails per workflow (default: 10)
        ORIONX_MAX_STEPS: Max steps per workflow (default: 100)
        ORIONX_WORKFLOW_TIMEOUT: Workflow timeout in seconds (default: 300)
        ORIONX_STEP_TIMEOUT: Step timeout in seconds (default: 30)
        ORIONX_PERSISTENCE_BACKEND: Persistence backend (memory|sqlite|redis)
        ORIONX_SQLITE_PATH: SQLite database path (default: ./orionx_executions.db)
        ORIONX_REDIS_URL: Redis URL (e.g., redis://localhost:6379/0)
        ORIONX_DEBUG: Enable debug mode (default: false)
        ORIONX_LOG_LEVEL: Log level (default: INFO)
    """
    limits = ExecutionLimits(
        max_db_queries=int(os.getenv("ORIONX_MAX_DB_QUERIES", "100")),
        max_api_calls=int(os.getenv("ORIONX_MAX_API_CALLS", "10")),
        max_emails=int(os.getenv("ORIONX_MAX_EMAILS", "10")),
        max_steps=int(os.getenv("ORIONX_MAX_STEPS", "100")),
        workflow_timeout_seconds=float(os.getenv("ORIONX_WORKFLOW_TIMEOUT", "300")),
        step_timeout_seconds=float(os.getenv("ORIONX_STEP_TIMEOUT", "30")),
    )
    
    backend_str = os.getenv("ORIONX_PERSISTENCE_BACKEND", "sqlite").lower()
    try:
        backend = PersistenceBackend(backend_str)
    except ValueError:
        backend = PersistenceBackend.SQLITE
    
    persistence = PersistenceConfig(
        backend=backend,
        sqlite_path=os.getenv("ORIONX_SQLITE_PATH", "./orionx_executions.db"),
        redis_url=os.getenv("ORIONX_REDIS_URL"),
        redis_prefix=os.getenv("ORIONX_REDIS_PREFIX", "orionx:"),
        redis_ttl_seconds=int(os.getenv("ORIONX_REDIS_TTL", str(86400 * 7))),
    )
    
    return OrionXConfig(
        limits=limits,
        persistence=persistence,
        debug=os.getenv("ORIONX_DEBUG", "false").lower() in ("true", "1", "yes"),
        log_level=os.getenv("ORIONX_LOG_LEVEL", "INFO").upper(),
    )


# Singleton config instance
_config: Optional[OrionXConfig] = None


def get_config() -> OrionXConfig:
    """Get the global configuration (lazy-loaded singleton)."""
    global _config
    if _config is None:
        _config = load_config()
    return _config


def reset_config() -> None:
    """Reset configuration (useful for testing)."""
    global _config
    _config = None
