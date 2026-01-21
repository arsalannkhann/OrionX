"""
OneX Workflow Schema

Workflow definitions for backend execution - no UI triggers.
Refactored from OrionX to remove element_event, page_load, and other UI concepts.
"""

from __future__ import annotations
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field, field_validator
from enum import Enum
from uuid import uuid4


# =============================================================================
# Trigger Types (Backend Only)
# =============================================================================

class TriggerType(str, Enum):
    """
    Types of workflow triggers.
    
    REMOVED (UI-specific):
    - ELEMENT_EVENT (UI clicks, changes)
    - PAGE_LOAD (UI lifecycle)
    """
    DATA_EVENT = "data_event"      # Database change event
    SCHEDULED = "scheduled"        # Time-based (cron)
    API_WEBHOOK = "api_webhook"    # External API call
    WORKFLOW_CALL = "workflow_call"  # Called by another workflow
    MANUAL = "manual"              # Programmatic trigger


class ErrorStrategy(str, Enum):
    """Error handling strategy for workflow steps."""
    STOP = "stop"          # Stop workflow on error
    CONTINUE = "continue"  # Continue to next step
    RETRY = "retry"        # Retry the step


class WorkflowTrigger(BaseModel):
    """Workflow trigger configuration."""
    type: TriggerType
    
    # For data_event
    entity_type: Optional[str] = None
    event_type: Optional[str] = None  # create, update, delete
    
    # For scheduled
    cron_expression: Optional[str] = None
    
    # For api_webhook
    webhook_path: Optional[str] = None
    http_method: Optional[str] = "POST"


# =============================================================================
# Step Types (Server-Side Only)
# =============================================================================

class StepType(str, Enum):
    """
    Types of workflow steps.
    
    REMOVED (Client-side):
    - set_state (UI state)
    - navigate (page navigation)
    - show_alert (UI alerts)
    - scroll_to (UI scrolling)
    - focus_element (UI focus)
    - reset_inputs (form reset)
    """
    # Data operations
    CREATE_ENTITY = "create_entity"
    UPDATE_ENTITY = "update_entity"
    DELETE_ENTITY = "delete_entity"
    QUERY_ENTITY = "query_entity"
    
    # External calls
    API_CALL = "api_call"
    SEND_EMAIL = "send_email"
    
    # Control flow
    CONDITION = "condition"
    LOOP = "loop"
    
    # Workflow operations
    SCHEDULE_WORKFLOW = "schedule_workflow"
    CALL_WORKFLOW = "call_workflow"
    
    # Data transformation
    TRANSFORM_DATA = "transform_data"
    VALIDATE_DATA = "validate_data"
    
    # Plugin
    PLUGIN_ACTION = "plugin_action"
    
    # State management
    SET_EXECUTION_STATE = "set_execution_state"
    
    # Logging
    LOG = "log"


# =============================================================================
# Workflow Step
# =============================================================================

class WorkflowStep(BaseModel):
    """
    A single step within a workflow.
    
    Steps form a DAG via dependencies, enabling parallel execution.
    Replaces WorkflowAction with execution-oriented naming.
    """
    uid: str = Field(default_factory=lambda: f"step_{uuid4().hex[:8]}")
    type: StepType
    name: Optional[str] = None
    description: Optional[str] = None
    params: Dict[str, Any] = Field(default_factory=dict)
    
    # Dependencies (other step UIDs that must complete first)
    depends_on: List[str] = Field(default_factory=list)
    
    # Conditional execution
    only_when: Optional[str] = None  # Expression that must be true
    
    # Error handling
    on_error: ErrorStrategy = ErrorStrategy.STOP
    
    # Timeout (milliseconds)
    timeout_ms: int = Field(default=30000, ge=100, le=300000)
    
    # Retry configuration
    max_retries: int = Field(default=0, ge=0, le=5)
    retry_delay_ms: int = Field(default=1000, ge=100, le=60000)
    
    @field_validator("uid")
    @classmethod
    def validate_uid(cls, v: str) -> str:
        if not v.startswith("step_"):
            raise ValueError("Step UID must start with 'step_'")
        return v


# =============================================================================
# Main Workflow Schema
# =============================================================================

class Workflow(BaseModel):
    """
    OneX Workflow Schema.
    
    A workflow is a sequence of steps triggered by an event.
    Steps form a DAG (Directed Acyclic Graph) via dependencies.
    
    Refactored from OrionXWorkflow to remove UI concerns.
    """
    uid: str = Field(default_factory=lambda: f"wf_{uuid4().hex[:8]}")
    name: str = Field(..., min_length=1, max_length=255)
    description: Optional[str] = Field(default=None, max_length=1000)
    version: int = Field(default=1, ge=1)
    
    # Trigger
    trigger: WorkflowTrigger
    
    # Steps (ordered by dependency resolution)
    steps: List[WorkflowStep] = Field(default_factory=list)
    
    # Variables (initial values)
    variables: Dict[str, Any] = Field(default_factory=dict)
    
    # Global settings
    enabled: bool = True
    
    # Concurrency control
    max_concurrent: int = Field(default=1, ge=1, le=100)
    
    # Timeout for entire workflow (milliseconds)
    timeout_ms: int = Field(default=300000, ge=1000, le=3600000)  # 5 min default, 1 hour max
    
    @field_validator("uid")
    @classmethod
    def validate_uid(cls, v: str) -> str:
        if not v.startswith("wf_"):
            raise ValueError("Workflow UID must start with 'wf_'")
        return v
    
    def get_execution_order(self) -> List[WorkflowStep]:
        """
        Topologically sort steps by dependencies.
        
        Returns:
            List of steps in execution order
            
        Raises:
            ValueError: If circular dependency detected
        """
        step_map = {s.uid: s for s in self.steps}
        in_degree = {s.uid: len(s.depends_on) for s in self.steps}
        dependents: Dict[str, List[str]] = {s.uid: [] for s in self.steps}
        
        for step in self.steps:
            for dep in step.depends_on:
                if dep in dependents:
                    dependents[dep].append(step.uid)
        
        # Kahn's algorithm
        queue = [uid for uid, deg in in_degree.items() if deg == 0]
        result = []
        
        while queue:
            uid = queue.pop(0)
            result.append(step_map[uid])
            
            for dependent in dependents[uid]:
                in_degree[dependent] -= 1
                if in_degree[dependent] == 0:
                    queue.append(dependent)
        
        if len(result) != len(self.steps):
            raise ValueError("Circular dependency detected in workflow steps")
        
        return result
    
    def to_ir(self) -> Dict[str, Any]:
        """Convert workflow to IR format for compilation."""
        return {
            "uid": self.uid,
            "version": self.version,
            "trigger": {
                "type": self.trigger.type.value,
            },
            "nodes": [
                {
                    "uid": step.uid,
                    "type": step.type.value,
                    "config": step.params,
                }
                for step in self.steps
            ],
            "edges": [
                {
                    "uid": f"edge_{step.uid}_{dep}",
                    "source": dep,
                    "target": step.uid,
                }
                for step in self.steps
                for dep in step.depends_on
            ],
            "variables": self.variables,
        }
