"""
OrionX Engine - Pure Backend Workflow Execution Engine

OrionX is a horizontal execution engine whose ONLY responsibility is:
- Executing workflows
- Managing execution state
- Enforcing permissions
- Calling external (vertical SaaS) APIs

OrionX does NOT:
- Generate UI
- Generate HTML / React
- Depend on frontend schemas
- Depend on any LLM, UI prompt, or UI-specific runtime
"""

__version__ = "0.1.0"

from .core.executor import WorkflowExecutor, OneXEngine as OrionXEngine
from .core.execution_logger import ExecutionLog, StepLog
from .schemas.execution import ExecutionContext, ExecutionStatus, StepResult, WorkflowResult
from .schemas.workflow import (
    Workflow,
    WorkflowStep,
    StepType,
    TriggerType,
)
from .permissions.permission_gate import PermissionGate, PermissionResult
from .state.state_store import StateStore, StateScope

# Alias for backward compatibility
OneXEngine = OrionXEngine

__all__ = [
    # Engine
    "OrionXEngine",
    "OneXEngine",  # Alias
    "WorkflowExecutor",
    # Execution
    "ExecutionLog",
    "StepLog",
    "ExecutionStatus",
    "ExecutionContext",
    "StepResult",
    "WorkflowResult",
    # Workflow
    "Workflow",
    "WorkflowStep",
    "StepType",
    "TriggerType",
    # Permissions
    "PermissionGate",
    "PermissionResult",
    # State
    "StateStore",
    "StateScope",
]
