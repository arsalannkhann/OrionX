"""
OneX API Routes

FastAPI endpoints for the OneX execution engine.
Implements the public API contract:
- SubmitWorkflow
- CancelWorkflow
- RetryWorkflow
- QueryExecution
"""

from __future__ import annotations
from typing import Dict, Any, Optional, List
from fastapi import APIRouter, HTTPException, BackgroundTasks, Depends
from pydantic import BaseModel, Field
from datetime import datetime

from ..core.executor import OneXEngine
from ..schemas.execution import ExecutionContext
from ..schemas.workflow import Workflow, TriggerType


# =============================================================================
# API Models
# =============================================================================

class SubmitWorkflowRequest(BaseModel):
    """Request to submit a workflow for execution."""
    workflow: Dict[str, Any] = Field(..., description="Workflow definition as JSON")
    context: Optional[Dict[str, Any]] = Field(default=None, description="Execution context")
    user_uid: Optional[str] = Field(default=None, description="User UID")
    
    class Config:
        json_schema_extra = {
            "example": {
                "workflow": {
                    "uid": "wf_example",
                    "name": "Example Workflow",
                    "version": 1,
                    "trigger": {"type": "manual"},
                    "steps": [
                        {
                            "uid": "step_log",
                            "type": "log",
                            "params": {"message": "Hello from OneX"}
                        }
                    ]
                },
                "context": {"input_params": {"id": "123"}},
                "user_uid": "user_abc"
            }
        }


class SubmitWorkflowResponse(BaseModel):
    """Response after submitting a workflow."""
    execution_id: str = Field(..., description="Unique execution ID")
    status: str = Field(default="pending", description="Initial status")
    submitted_at: str = Field(..., description="ISO timestamp")


class ExecutionStatusResponse(BaseModel):
    """Response for execution status query."""
    execution_id: str
    workflow_uid: str
    user_uid: Optional[str]
    status: str
    started_at: str
    completed_at: Optional[str]
    steps: List[Dict[str, Any]]
    output: Optional[Any]
    error: Optional[Dict[str, Any]]


class CancelWorkflowResponse(BaseModel):
    """Response after cancelling a workflow."""
    execution_id: str
    cancelled: bool
    message: str


class RetryWorkflowResponse(BaseModel):
    """Response after retrying a workflow."""
    original_execution_id: str
    new_execution_id: Optional[str]
    message: str


class HealthResponse(BaseModel):
    """Health check response."""
    status: str
    version: str
    timestamp: str


class StepTypeInfo(BaseModel):
    """Information about a step type."""
    name: str
    description: str


class TriggerTypeInfo(BaseModel):
    """Information about a trigger type."""
    name: str
    description: str


class SchemaInfoResponse(BaseModel):
    """Response with schema information."""
    step_types: List[StepTypeInfo]
    trigger_types: List[TriggerTypeInfo]


# =============================================================================
# Router
# =============================================================================

router = APIRouter(prefix="/api/v1", tags=["OneX Execution Engine"])

# Engine singleton (will be injected in production)
_engine: Optional[OneXEngine] = None


def get_engine() -> OneXEngine:
    """Get or create the OneX engine instance."""
    global _engine
    if _engine is None:
        _engine = OneXEngine()
    return _engine


# =============================================================================
# Endpoints
# =============================================================================

@router.get(
    "/health",
    response_model=HealthResponse,
    summary="Health Check",
    description="Check if the OneX engine is running.",
)
async def health_check():
    """Health check endpoint."""
    return HealthResponse(
        status="healthy",
        version="0.1.0",
        timestamp=datetime.utcnow().isoformat(),
    )


@router.get(
    "/schema",
    response_model=SchemaInfoResponse,
    summary="Get Schema Info",
    description="Get available step types and trigger types.",
)
async def get_schema_info():
    """Get schema information."""
    from ..schemas.workflow import StepType
    
    step_docs = {
        "create_entity": "Create a database entity",
        "update_entity": "Update a database entity",
        "delete_entity": "Delete a database entity",
        "query_entity": "Query database entities",
        "api_call": "Make an external API call",
        "send_email": "Send an email",
        "condition": "Conditional branching",
        "loop": "Loop over a collection",
        "schedule_workflow": "Schedule a workflow for later",
        "call_workflow": "Call another workflow",
        "transform_data": "Transform data using a mapping",
        "validate_data": "Validate data against rules",
        "plugin_action": "Execute a plugin action",
        "set_execution_state": "Set execution state",
        "log": "Log a message",
    }
    
    trigger_docs = {
        "data_event": "Triggered by database change",
        "scheduled": "Triggered by schedule (cron)",
        "api_webhook": "Triggered by external API call",
        "workflow_call": "Triggered by another workflow",
        "manual": "Triggered programmatically",
    }
    
    return SchemaInfoResponse(
        step_types=[
            StepTypeInfo(name=t.value, description=step_docs.get(t.value, ""))
            for t in StepType
        ],
        trigger_types=[
            TriggerTypeInfo(name=t.value, description=trigger_docs.get(t.value, ""))
            for t in TriggerType
        ],
    )


@router.post(
    "/workflows/submit",
    response_model=SubmitWorkflowResponse,
    summary="Submit Workflow",
    description="Submit a workflow for execution. Returns immediately with an execution ID.",
)
async def submit_workflow(
    request: SubmitWorkflowRequest,
    background_tasks: BackgroundTasks,
    engine: OneXEngine = Depends(get_engine),
):
    """
    Submit a workflow for execution.
    
    The workflow is executed asynchronously. Use the returned execution_id
    to query the status.
    """
    try:
        # Parse workflow
        workflow = Workflow(**request.workflow)
        
        # Build context
        context = ExecutionContext()
        if request.context:
            context.input_params = request.context.get("input_params", {})
            context.execution_state = request.context.get("execution_state", {})
            if "user" in request.context:
                context.user = request.context["user"]
        
        # Execute (synchronous for now, can be made async with background_tasks)
        execution_id = await engine.submit_workflow(
            workflow,
            context,
            request.user_uid,
        )
        
        return SubmitWorkflowResponse(
            execution_id=execution_id,
            status="running",
            submitted_at=datetime.utcnow().isoformat(),
        )
        
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get(
    "/executions/{execution_id}",
    response_model=ExecutionStatusResponse,
    summary="Query Execution",
    description="Get the status and results of a workflow execution.",
)
async def query_execution(
    execution_id: str,
    engine: OneXEngine = Depends(get_engine),
):
    """Query execution status by ID."""
    result = engine.query_execution(execution_id)
    
    if result is None:
        raise HTTPException(status_code=404, detail="Execution not found")
    
    return ExecutionStatusResponse(
        execution_id=result.execution_id,
        workflow_uid=result.workflow_uid,
        user_uid=result.user_uid,
        status=result.status.value,
        started_at=result.started_at.isoformat(),
        completed_at=result.completed_at.isoformat() if result.completed_at else None,
        steps=[
            {
                "step_uid": s.step_uid,
                "step_type": s.step_type,
                "success": s.success,
                "duration_ms": s.duration_ms,
                "error": s.error,
            }
            for s in result.steps
        ],
        output=result.output,
        error=result.error,
    )


@router.post(
    "/executions/{execution_id}/cancel",
    response_model=CancelWorkflowResponse,
    summary="Cancel Workflow",
    description="Cancel a running workflow execution.",
)
async def cancel_workflow(
    execution_id: str,
    engine: OneXEngine = Depends(get_engine),
):
    """Cancel a running workflow."""
    cancelled = await engine.cancel_workflow(execution_id)
    
    return CancelWorkflowResponse(
        execution_id=execution_id,
        cancelled=cancelled,
        message="Workflow cancelled" if cancelled else "Workflow not found or already completed",
    )


@router.post(
    "/executions/{execution_id}/retry",
    response_model=RetryWorkflowResponse,
    summary="Retry Workflow",
    description="Retry a failed workflow execution.",
)
async def retry_workflow(
    execution_id: str,
    engine: OneXEngine = Depends(get_engine),
):
    """Retry a failed workflow."""
    try:
        new_execution_id = await engine.retry_workflow(execution_id)
        
        if new_execution_id:
            return RetryWorkflowResponse(
                original_execution_id=execution_id,
                new_execution_id=new_execution_id,
                message="Workflow retry started",
            )
        else:
            return RetryWorkflowResponse(
                original_execution_id=execution_id,
                new_execution_id=None,
                message="Original execution not found",
            )
    except NotImplementedError as e:
        raise HTTPException(status_code=501, detail=str(e))


# =============================================================================
# Event Webhooks (Outbound Events)
# =============================================================================

class WebhookConfig(BaseModel):
    """Webhook configuration for receiving events."""
    url: str
    events: List[str] = Field(
        default=["all"],
        description="Events to subscribe to: execution_started, step_completed, step_failed, workflow_completed, workflow_failed, all"
    )
    secret: Optional[str] = Field(default=None, description="Secret for webhook signature")


# Event documentation (not actual endpoints, just for OpenAPI docs)
EVENT_DESCRIPTIONS = {
    "ExecutionStarted": {
        "description": "Emitted when workflow execution starts",
        "payload": {
            "event": "execution_started",
            "execution_id": "exec_abc123",
            "workflow_uid": "wf_example",
            "user_uid": "user_123",
            "timestamp": "2024-01-01T00:00:00Z"
        }
    },
    "StepCompleted": {
        "description": "Emitted when a step completes successfully",
        "payload": {
            "event": "step_completed",
            "execution_id": "exec_abc123",
            "step_uid": "step_001",
            "result": {"data": "..."},
            "duration_ms": 150,
            "timestamp": "2024-01-01T00:00:01Z"
        }
    },
    "StepFailed": {
        "description": "Emitted when a step fails",
        "payload": {
            "event": "step_failed",
            "execution_id": "exec_abc123",
            "step_uid": "step_001",
            "error": "Connection timeout",
            "timestamp": "2024-01-01T00:00:01Z"
        }
    },
    "WorkflowCompleted": {
        "description": "Emitted when workflow completes successfully",
        "payload": {
            "event": "workflow_completed",
            "execution_id": "exec_abc123",
            "status": "completed",
            "output": {"result": "..."},
            "timestamp": "2024-01-01T00:00:05Z"
        }
    },
    "WorkflowFailed": {
        "description": "Emitted when workflow fails",
        "payload": {
            "event": "workflow_failed",
            "execution_id": "exec_abc123",
            "error": "Step step_001 failed",
            "failed_step": "step_001",
            "timestamp": "2024-01-01T00:00:05Z"
        }
    },
}
