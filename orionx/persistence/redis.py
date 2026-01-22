"""
Redis Execution Store

Distributed persistent storage for production use.
For multi-server deployments and high availability.
"""

from typing import List, Optional
from datetime import datetime
import json
import logging

from .base import ExecutionStore, ExecutionRecord
from ..schemas.execution import ExecutionStatus

logger = logging.getLogger(__name__)


class RedisExecutionStore(ExecutionStore):
    """
    Redis-backed execution store.
    
    Features:
    - Distributed persistence
    - TTL-based automatic cleanup
    - Atomic operations
    - High performance
    
    Use for:
    - Multi-server production deployments
    - High-availability requirements
    - Large-scale workflow processing
    
    Requires:
    - redis package: pip install redis
    """
    
    def __init__(
        self,
        url: Optional[str] = None,
        prefix: str = "orionx:",
        ttl: int = 86400 * 7,  # 7 days
    ):
        self.prefix = prefix
        self.ttl = ttl
        self._redis = None
        self._url = url or "redis://localhost:6379/0"
        
    def _get_redis(self):
        """Lazy-load Redis connection."""
        if self._redis is None:
            try:
                import redis.asyncio as redis
                self._redis = redis.from_url(self._url)
            except ImportError:
                raise ImportError(
                    "Redis support requires the 'redis' package. "
                    "Install with: pip install redis"
                )
        return self._redis
    
    def _key(self, execution_id: str) -> str:
        """Generate Redis key for execution."""
        return f"{self.prefix}execution:{execution_id}"
    
    def _status_key(self, status: ExecutionStatus) -> str:
        """Generate Redis key for status index."""
        return f"{self.prefix}status:{status.value}"
    
    async def save(self, record: ExecutionRecord) -> None:
        """Save or update an execution record."""
        redis = self._get_redis()
        key = self._key(record.execution_id)
        
        # Store as JSON
        data = json.dumps(record.to_dict())
        await redis.setex(key, self.ttl, data)
        
        # Update status index
        status_key = self._status_key(record.status)
        await redis.sadd(status_key, record.execution_id)
    
    async def get(self, execution_id: str) -> Optional[ExecutionRecord]:
        """Get an execution record by ID."""
        redis = self._get_redis()
        key = self._key(execution_id)
        
        data = await redis.get(key)
        if data:
            return ExecutionRecord.from_dict(json.loads(data))
        return None
    
    async def delete(self, execution_id: str) -> bool:
        """Delete an execution record."""
        redis = self._get_redis()
        key = self._key(execution_id)
        
        # Get current status to remove from index
        record = await self.get(execution_id)
        if record:
            status_key = self._status_key(record.status)
            await redis.srem(status_key, execution_id)
        
        result = await redis.delete(key)
        return result > 0
    
    async def list_by_status(self, status: ExecutionStatus) -> List[ExecutionRecord]:
        """List executions with a given status."""
        redis = self._get_redis()
        status_key = self._status_key(status)
        
        execution_ids = await redis.smembers(status_key)
        records = []
        
        for exec_id in execution_ids:
            if isinstance(exec_id, bytes):
                exec_id = exec_id.decode()
            record = await self.get(exec_id)
            if record:
                records.append(record)
            else:
                # Clean up stale index entry
                await redis.srem(status_key, exec_id)
        
        return records
    
    async def list_active(self) -> List[ExecutionRecord]:
        """List all active (running/pending) executions."""
        pending = await self.list_by_status(ExecutionStatus.PENDING)
        running = await self.list_by_status(ExecutionStatus.RUNNING)
        return pending + running
    
    async def update_status(
        self,
        execution_id: str,
        status: ExecutionStatus,
        error: Optional[dict] = None,
        completed_at: Optional[datetime] = None,
    ) -> bool:
        """Update execution status."""
        record = await self.get(execution_id)
        if not record:
            return False
        
        redis = self._get_redis()
        
        # Remove from old status index
        old_status_key = self._status_key(record.status)
        await redis.srem(old_status_key, execution_id)
        
        # Update record
        record.status = status
        if error is not None:
            record.error = error
        if completed_at is not None:
            record.completed_at = completed_at
        
        # Save updated record
        await self.save(record)
        
        return True
    
    async def append_step_log(
        self,
        execution_id: str,
        step_log: dict,
    ) -> bool:
        """Append a step log to an execution."""
        record = await self.get(execution_id)
        if not record:
            return False
        
        record.step_logs.append(step_log)
        await self.save(record)
        
        return True
