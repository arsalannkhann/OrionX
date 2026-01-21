"""
OneX Execution Schemas

Pure backend execution types - no UI concepts.
"""

from __future__ import annotations
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum


# =============================================================================
# Execution Status
# =============================================================================

class ExecutionStatus(str, Enum):
    """Status of a workflow execution."""
    PENDING = "pending"
    RUNNING = "running"
    PAUSED = "paused"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"
    CANCELLED = "cancelled"


# =============================================================================
# Execution Context
# =============================================================================

@dataclass
class ExecutionContext:
    """
    Context for workflow execution.
    
    This replaces EvaluationContext with execution-oriented naming:
    - No page_state (use execution_state)
    - No url_params (use input_params)
    - No current_thing_type (use entity_type)
    """
    # User context (who is executing)
    user: Optional[Dict[str, Any]] = None
    
    # Execution state (replaces page_state)
    execution_state: Dict[str, Any] = field(default_factory=dict)
    
    # Input parameters (replaces url_params)
    input_params: Dict[str, Any] = field(default_factory=dict)
    
    # Workflow data (step results)
    workflow_data: Dict[str, Any] = field(default_factory=dict)
    
    # Current entity being processed
    current_entity: Optional[Dict[str, Any]] = None
    entity_type: Optional[str] = None
    
    # Parent context (for nested workflows)
    parent: Optional["ExecutionContext"] = None
    
    def get(self, name: str) -> Any:
        """Get a named value from context."""
        # Check built-in contexts
        if name == "Current User":
            return self.user
        if name.startswith("This "):
            return self.current_entity
        
        # Check execution state
        if name in self.execution_state:
            return self.execution_state[name]
        
        # Check input params
        if name in self.input_params:
            return self.input_params[name]
        
        # Check workflow data
        if name in self.workflow_data:
            return self.workflow_data[name]
        
        # Check parent
        if self.parent:
            return self.parent.get(name)
        
        return None
    
    def with_entity(self, entity: Dict[str, Any], entity_type: str) -> "ExecutionContext":
        """Create child context with current entity set."""
        return ExecutionContext(
            user=self.user,
            execution_state=self.execution_state.copy(),
            input_params=self.input_params,
            workflow_data=self.workflow_data,
            current_entity=entity,
            entity_type=entity_type,
            parent=self,
        )
    
    def set_state(self, key: str, value: Any) -> None:
        """Set execution state value."""
        self.execution_state[key] = value
    
    def set_result(self, step_uid: str, result: Any) -> None:
        """Set step result in workflow data."""
        self.workflow_data[step_uid] = result


# =============================================================================
# Step Result
# =============================================================================

@dataclass
class StepResult:
    """
    Result of a single step execution.
    
    Replaces ActionLog with execution-oriented naming.
    """
    step_uid: str
    step_type: str
    started_at: datetime
    completed_at: Optional[datetime] = None
    duration_ms: Optional[int] = None
    inputs: Dict[str, Any] = field(default_factory=dict)
    result: Optional[Any] = None
    error: Optional[str] = None
    skipped: bool = False
    skip_reason: Optional[str] = None
    
    @property
    def success(self) -> bool:
        """Check if step completed successfully."""
        return self.error is None and not self.skipped


# =============================================================================
# Workflow Result
# =============================================================================

@dataclass
class WorkflowResult:
    """
    Final result of workflow execution.
    
    This is the API response format for QueryExecution.
    """
    execution_id: str
    workflow_uid: str
    user_uid: Optional[str]
    started_at: datetime
    completed_at: Optional[datetime] = None
    status: ExecutionStatus = ExecutionStatus.PENDING
    steps: List[StepResult] = field(default_factory=list)
    output: Optional[Any] = None
    error: Optional[Dict[str, Any]] = None
    input_snapshot: Optional[Dict] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for API response."""
        return {
            "execution_id": self.execution_id,
            "workflow_uid": self.workflow_uid,
            "user_uid": self.user_uid,
            "started_at": self.started_at.isoformat(),
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "status": self.status.value,
            "steps": [
                {
                    "step_uid": s.step_uid,
                    "step_type": s.step_type,
                    "success": s.success,
                    "duration_ms": s.duration_ms,
                    "error": s.error,
                    "skipped": s.skipped,
                }
                for s in self.steps
            ],
            "output": self.output,
            "error": self.error,
        }


# =============================================================================
# Execution Events
# =============================================================================

@dataclass
class ExecutionEvent:
    """Base class for execution events."""
    execution_id: str
    timestamp: datetime = field(default_factory=datetime.utcnow)


@dataclass
class ExecutionStartedEvent(ExecutionEvent):
    """Emitted when workflow execution starts."""
    workflow_uid: str = ""
    user_uid: Optional[str] = None


@dataclass
class StepCompletedEvent(ExecutionEvent):
    """Emitted when a step completes successfully."""
    step_uid: str = ""
    result: Optional[Any] = None
    duration_ms: int = 0


@dataclass
class StepFailedEvent(ExecutionEvent):
    """Emitted when a step fails."""
    step_uid: str = ""
    error: str = ""


@dataclass
class WorkflowCompletedEvent(ExecutionEvent):
    """Emitted when workflow completes successfully."""
    status: ExecutionStatus = ExecutionStatus.COMPLETED
    output: Optional[Any] = None


@dataclass
class WorkflowFailedEvent(ExecutionEvent):
    """Emitted when workflow fails."""
    error: str = ""
    failed_step: Optional[str] = None
