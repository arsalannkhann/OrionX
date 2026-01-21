"""
OneX Execution Plan

Structures for compiled execution plans.
Copied from OrionX with minimal changes.
"""

from __future__ import annotations
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from enum import Enum


# =============================================================================
# Validation
# =============================================================================

class ValidationSeverity(str, Enum):
    """Severity of validation issues."""
    BLOCKING = "blocking"
    WARNING = "warning"
    INFO = "info"


@dataclass
class ValidationIssue:
    """A single validation issue."""
    code: str
    severity: ValidationSeverity
    message: str
    nodes: Optional[List[str]] = None
    edges: Optional[List[str]] = None


@dataclass
class ValidationResult:
    """Result of workflow validation."""
    valid: bool
    errors: List[ValidationIssue] = field(default_factory=list)
    warnings: List[ValidationIssue] = field(default_factory=list)
    
    def has_blocking(self) -> bool:
        """Check if there are blocking errors."""
        return any(e.severity == ValidationSeverity.BLOCKING for e in self.errors)


# =============================================================================
# Execution Plan
# =============================================================================

@dataclass
class ExecutionGroup:
    """Group of steps that can execute in parallel."""
    depth: int
    node_uids: List[str] = field(default_factory=list)


@dataclass
class ExecutionPlan:
    """
    Compiled execution plan for a workflow.
    
    Contains parallel execution groups and validation results.
    """
    workflow_uid: str
    version: int
    groups: List[ExecutionGroup] = field(default_factory=list)
    variable_bindings: Dict[str, Any] = field(default_factory=dict)
    validation: Optional[ValidationResult] = None
    compiled_at: str = ""
    compiler_version: str = "1.0.0"
    
    @property
    def total_steps(self) -> int:
        """Get total number of steps."""
        return sum(len(g.node_uids) for g in self.groups)
    
    @property
    def max_parallelism(self) -> int:
        """Get maximum parallel steps."""
        return max(len(g.node_uids) for g in self.groups) if self.groups else 0


# =============================================================================
# Compilation Error
# =============================================================================

class CompilationError(Exception):
    """Error during workflow compilation."""
    
    def __init__(
        self,
        code: str,
        message: str,
        nodes: Optional[List[str]] = None,
        edges: Optional[List[str]] = None,
    ):
        super().__init__(message)
        self.code = code
        self.nodes = nodes
        self.edges = edges
