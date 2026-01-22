"""
In-Memory Execution Store

Fast, non-persistent storage for testing and development.
"""

from typing import Dict, List, Optional
from datetime import datetime

from .base import ExecutionStore, ExecutionRecord
from ..schemas.execution import ExecutionStatus


class InMemoryExecutionStore(ExecutionStore):
    """
    In-memory execution store (no persistence).
    
    Use for:
    - Unit testing
    - Development
    - Short-lived workflows
    
    WARNING: All data is lost on server restart.
    """
    
    def __init__(self):
        self._records: Dict[str, ExecutionRecord] = {}
    
    async def save(self, record: ExecutionRecord) -> None:
        """Save or update an execution record."""
        self._records[record.execution_id] = record
    
    async def get(self, execution_id: str) -> Optional[ExecutionRecord]:
        """Get an execution record by ID."""
        return self._records.get(execution_id)
    
    async def delete(self, execution_id: str) -> bool:
        """Delete an execution record."""
        if execution_id in self._records:
            del self._records[execution_id]
            return True
        return False
    
    async def list_by_status(self, status: ExecutionStatus) -> List[ExecutionRecord]:
        """List executions with a given status."""
        return [r for r in self._records.values() if r.status == status]
    
    async def list_active(self) -> List[ExecutionRecord]:
        """List all active (running/pending) executions."""
        active_statuses = {ExecutionStatus.PENDING, ExecutionStatus.RUNNING}
        return [r for r in self._records.values() if r.status in active_statuses]
    
    async def update_status(
        self,
        execution_id: str,
        status: ExecutionStatus,
        error: Optional[dict] = None,
        completed_at: Optional[datetime] = None,
    ) -> bool:
        """Update execution status."""
        record = self._records.get(execution_id)
        if record:
            record.status = status
            if error is not None:
                record.error = error
            if completed_at is not None:
                record.completed_at = completed_at
            return True
        return False
    
    async def append_step_log(
        self,
        execution_id: str,
        step_log: dict,
    ) -> bool:
        """Append a step log to an execution."""
        record = self._records.get(execution_id)
        if record:
            record.step_logs.append(step_log)
            return True
        return False
    
    def clear(self) -> None:
        """Clear all records (for testing)."""
        self._records.clear()
