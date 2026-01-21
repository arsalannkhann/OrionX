"""
OneX Execution Logger

Execution logging, tracing, and replay capabilities.
Copied from OrionX with minimal UI-related changes.
"""

from __future__ import annotations
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from datetime import datetime
import logging

from ..schemas.execution import ExecutionStatus, StepResult


logger = logging.getLogger(__name__)


# =============================================================================
# Step Log (replaces ActionLog)
# =============================================================================

@dataclass
class StepLog:
    """Log entry for a single step execution."""
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


@dataclass
class ExecutionLog:
    """Complete log for a workflow execution."""
    execution_id: str
    workflow_uid: str
    user_uid: Optional[str]
    started_at: datetime
    completed_at: Optional[datetime] = None
    status: ExecutionStatus = ExecutionStatus.PENDING
    step_logs: List[StepLog] = field(default_factory=list)
    error: Optional[Dict[str, Any]] = None
    input_snapshot: Optional[Dict] = None


# =============================================================================
# Expression Trace
# =============================================================================

@dataclass
class ExpressionStep:
    """Single step in expression evaluation."""
    step_number: int
    node_type: str
    input_repr: str
    output_value: Any
    output_repr: str
    duration_us: int


@dataclass
class ExpressionTrace:
    """Full trace of expression evaluation."""
    expression_raw: str
    expression_uid: Optional[str]
    steps: List[ExpressionStep] = field(default_factory=list)
    final_result: Any = None
    final_type: str = "unknown"
    total_duration_ms: float = 0
    error: Optional[str] = None


class ExpressionTracer:
    """Traces expression evaluation for debugging."""
    
    def __init__(self):
        self._traces: Dict[str, ExpressionTrace] = {}
        self._current: Optional[ExpressionTrace] = None
        self._start_time: Optional[float] = None
    
    def start(self, expr_raw: str, expr_uid: Optional[str] = None):
        """Start tracing an expression."""
        import time
        self._start_time = time.perf_counter()
        self._current = ExpressionTrace(
            expression_raw=expr_raw,
            expression_uid=expr_uid,
        )
    
    def step(
        self,
        step_num: int,
        node_type: str,
        input_repr: str,
        output: Any,
        output_repr: str,
    ):
        """Record a step in evaluation."""
        if not self._current:
            return
        
        import time
        elapsed_us = int((time.perf_counter() - self._start_time) * 1_000_000)
        
        self._current.steps.append(ExpressionStep(
            step_number=step_num,
            node_type=node_type,
            input_repr=input_repr,
            output_value=output,
            output_repr=output_repr,
            duration_us=elapsed_us,
        ))
    
    def finish(self, result: Any) -> ExpressionTrace:
        """Finish tracing and return the trace."""
        if not self._current:
            return ExpressionTrace(expression_raw="", steps=[])
        
        import time
        self._current.total_duration_ms = (time.perf_counter() - self._start_time) * 1000
        self._current.final_result = result
        self._current.final_type = type(result).__name__
        
        trace = self._current
        self._current = None
        self._start_time = None
        
        return trace
    
    def error(self, message: str) -> ExpressionTrace:
        """Finish tracing with an error."""
        if not self._current:
            return ExpressionTrace(expression_raw="", error=message)
        
        self._current.error = message
        trace = self._current
        self._current = None
        
        return trace


# =============================================================================
# Workflow Debugger
# =============================================================================

@dataclass
class DebugBreakpoint:
    """Breakpoint for workflow debugging."""
    step_uid: str
    condition: Optional[str] = None
    hit_count: int = 0
    enabled: bool = True


@dataclass
class DebugSession:
    """Debugging session state."""
    session_id: str
    workflow_uid: str
    breakpoints: List[DebugBreakpoint] = field(default_factory=list)
    paused_at: Optional[str] = None
    step_mode: bool = False
    variable_watches: List[str] = field(default_factory=list)


class WorkflowDebugger:
    """Provides debugging capabilities for workflows."""
    
    def __init__(self):
        self._sessions: Dict[str, DebugSession] = {}
    
    def create_session(
        self,
        workflow_uid: str,
        breakpoints: Optional[List[str]] = None,
    ) -> str:
        """Create a new debug session."""
        import uuid
        session_id = f"debug_{uuid.uuid4().hex[:8]}"
        
        session = DebugSession(
            session_id=session_id,
            workflow_uid=workflow_uid,
            breakpoints=[
                DebugBreakpoint(step_uid=uid)
                for uid in (breakpoints or [])
            ],
        )
        
        self._sessions[session_id] = session
        return session_id
    
    def add_breakpoint(self, session_id: str, step_uid: str, condition: Optional[str] = None):
        """Add a breakpoint."""
        session = self._sessions.get(session_id)
        if session:
            session.breakpoints.append(DebugBreakpoint(
                step_uid=step_uid,
                condition=condition,
            ))
    
    def remove_breakpoint(self, session_id: str, step_uid: str):
        """Remove a breakpoint."""
        session = self._sessions.get(session_id)
        if session:
            session.breakpoints = [
                bp for bp in session.breakpoints
                if bp.step_uid != step_uid
            ]
    
    def check_breakpoint(
        self,
        session_id: str,
        step_uid: str,
        context: Dict[str, Any],
    ) -> bool:
        """Check if execution should pause at this step."""
        session = self._sessions.get(session_id)
        if not session:
            return False
        
        if session.step_mode:
            session.paused_at = step_uid
            return True
        
        for bp in session.breakpoints:
            if bp.step_uid == step_uid and bp.enabled:
                bp.hit_count += 1
                session.paused_at = step_uid
                return True
        
        return False
    
    def step(self, session_id: str):
        """Enable step mode and continue to next step."""
        session = self._sessions.get(session_id)
        if session:
            session.step_mode = True
            session.paused_at = None
    
    def continue_execution(self, session_id: str):
        """Continue execution until next breakpoint."""
        session = self._sessions.get(session_id)
        if session:
            session.step_mode = False
            session.paused_at = None
    
    def end_session(self, session_id: str):
        """End a debug session."""
        self._sessions.pop(session_id, None)


# =============================================================================
# Execution Log Storage Interface
# =============================================================================

class ExecutionLogStorage:
    """Abstract interface for execution log storage."""
    
    async def save_log(self, log: ExecutionLog) -> None:
        """Save an execution log."""
        raise NotImplementedError
    
    async def get_log(self, execution_id: str) -> Optional[ExecutionLog]:
        """Get an execution log by ID."""
        raise NotImplementedError
    
    async def get_logs_for_workflow(
        self,
        workflow_uid: str,
        limit: int = 100,
    ) -> List[ExecutionLog]:
        """Get recent logs for a workflow."""
        raise NotImplementedError
    
    async def get_logs_for_user(
        self,
        user_uid: str,
        limit: int = 100,
    ) -> List[ExecutionLog]:
        """Get recent logs for a user."""
        raise NotImplementedError


class InMemoryLogStorage(ExecutionLogStorage):
    """In-memory implementation for testing."""
    
    def __init__(self):
        self._logs: Dict[str, ExecutionLog] = {}
    
    async def save_log(self, log: ExecutionLog) -> None:
        self._logs[log.execution_id] = log
    
    async def get_log(self, execution_id: str) -> Optional[ExecutionLog]:
        return self._logs.get(execution_id)
    
    async def get_logs_for_workflow(
        self,
        workflow_uid: str,
        limit: int = 100,
    ) -> List[ExecutionLog]:
        return [
            log for log in self._logs.values()
            if log.workflow_uid == workflow_uid
        ][:limit]
    
    async def get_logs_for_user(
        self,
        user_uid: str,
        limit: int = 100,
    ) -> List[ExecutionLog]:
        return [
            log for log in self._logs.values()
            if log.user_uid == user_uid
        ][:limit]
