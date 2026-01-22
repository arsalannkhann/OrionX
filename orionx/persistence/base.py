"""
ExecutionStore Base Interface

Abstract interface for execution state persistence.
Addresses audit finding: "Zero persistence for workflow execution state"
"""

from abc import ABC, abstractmethod
from typing import List, Optional
from datetime import datetime

from ..schemas.execution import ExecutionStatus


class ExecutionRecord:
    """
    Serializable execution record for persistence.
    
    Separate from ExecutionLog to allow JSON serialization.
    """
    def __init__(
        self,
        execution_id: str,
        workflow_uid: str,
        user_uid: Optional[str],
        status: ExecutionStatus,
        started_at: datetime,
        completed_at: Optional[datetime] = None,
        step_logs: Optional[List[dict]] = None,
        error: Optional[dict] = None,
        input_snapshot: Optional[dict] = None,
        output: Optional[dict] = None,
    ):
        self.execution_id = execution_id
        self.workflow_uid = workflow_uid
        self.user_uid = user_uid
        self.status = status
        self.started_at = started_at
        self.completed_at = completed_at
        self.step_logs = step_logs or []
        self.error = error
        self.input_snapshot = input_snapshot
        self.output = output
    
    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        return {
            "execution_id": self.execution_id,
            "workflow_uid": self.workflow_uid,
            "user_uid": self.user_uid,
            "status": self.status.value,
            "started_at": self.started_at.isoformat(),
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "step_logs": self.step_logs,
            "error": self.error,
            "input_snapshot": self.input_snapshot,
            "output": self.output,
        }
    
    @classmethod
    def from_dict(cls, data: dict) -> "ExecutionRecord":
        """Create from dictionary."""
        return cls(
            execution_id=data["execution_id"],
            workflow_uid=data["workflow_uid"],
            user_uid=data.get("user_uid"),
            status=ExecutionStatus(data["status"]),
            started_at=datetime.fromisoformat(data["started_at"]),
            completed_at=datetime.fromisoformat(data["completed_at"]) if data.get("completed_at") else None,
            step_logs=data.get("step_logs", []),
            error=data.get("error"),
            input_snapshot=data.get("input_snapshot"),
            output=data.get("output"),
        )


class ExecutionStore(ABC):
    """
    Abstract interface for execution state persistence.
    
    Implementations:
    - InMemoryExecutionStore: For testing (no persistence)
    - SQLiteExecutionStore: File-based persistence (default)
    - RedisExecutionStore: Distributed persistence (production)
    """
    
    @abstractmethod
    async def save(self, record: ExecutionRecord) -> None:
        """Save or update an execution record."""
        pass
    
    @abstractmethod
    async def get(self, execution_id: str) -> Optional[ExecutionRecord]:
        """Get an execution record by ID."""
        pass
    
    @abstractmethod
    async def delete(self, execution_id: str) -> bool:
        """Delete an execution record. Returns True if deleted."""
        pass
    
    @abstractmethod
    async def list_by_status(self, status: ExecutionStatus) -> List[ExecutionRecord]:
        """List executions with a given status."""
        pass
    
    @abstractmethod
    async def list_active(self) -> List[ExecutionRecord]:
        """List all active (running/pending) executions."""
        pass
    
    @abstractmethod
    async def update_status(
        self,
        execution_id: str,
        status: ExecutionStatus,
        error: Optional[dict] = None,
        completed_at: Optional[datetime] = None,
    ) -> bool:
        """Update execution status. Returns True if updated."""
        pass
    
    @abstractmethod
    async def append_step_log(
        self,
        execution_id: str,
        step_log: dict,
    ) -> bool:
        """Append a step log to an execution. Returns True if updated."""
        pass
    
    async def list_for_workflow(self, workflow_uid: str, limit: int = 100) -> List[ExecutionRecord]:
        """List recent executions for a workflow."""
        # Default implementation - can be overridden for efficiency
        all_records = await self.list_active()
        return [r for r in all_records if r.workflow_uid == workflow_uid][:limit]
    
    async def list_for_user(self, user_uid: str, limit: int = 100) -> List[ExecutionRecord]:
        """List recent executions for a user."""
        all_records = await self.list_active()
        return [r for r in all_records if r.user_uid == user_uid][:limit]
