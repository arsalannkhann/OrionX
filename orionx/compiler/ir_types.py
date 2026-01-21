"""
OneX IR Types

Intermediate representation for compiled workflows.
Extracted from OrionX with UI-specific types removed.

REMOVED (UI-specific):
- IR_Page
- IR_Element
- IR_App.pages

KEPT:
- Opcode
- IR_Type
- IR_Instruction
- IR_Expr

ADDED:
- IR_Workflow
- IR_Step
"""

from __future__ import annotations
from typing import Dict, List, Any, Optional, Set
from dataclasses import dataclass, field
from enum import IntEnum, auto
from datetime import datetime


# =============================================================================
# Opcodes
# =============================================================================

class Opcode(IntEnum):
    """Compact instruction opcodes for IR."""
    # Stack operations
    PUSH_CONST = auto()
    POP = auto()
    DUP = auto()
    
    # Context loading
    LOAD_CONTEXT = auto()
    LOAD_STATE = auto()
    LOAD_PARAM = auto()
    
    # Property access
    GET_FIELD = auto()
    GET_INDEX = auto()
    
    # Arithmetic
    ADD = auto()
    SUB = auto()
    MUL = auto()
    DIV = auto()
    MOD = auto()
    NEG = auto()
    
    # Comparison
    EQ = auto()
    NEQ = auto()
    LT = auto()
    LTE = auto()
    GT = auto()
    GTE = auto()
    
    # Logical
    AND = auto()
    OR = auto()
    NOT = auto()
    
    # String operations
    CONCAT = auto()
    CONTAINS = auto()
    STARTS_WITH = auto()
    ENDS_WITH = auto()
    
    # Control flow
    JUMP = auto()
    JUMP_IF_TRUE = auto()
    JUMP_IF_FALSE = auto()
    
    # Functions
    CALL = auto()
    
    # Object creation
    MAKE_LIST = auto()
    MAKE_OBJECT = auto()
    
    # Special
    NOP = auto()
    HALT = auto()


# =============================================================================
# IR Types
# =============================================================================

class IR_Type(IntEnum):
    """Compact type codes for IR values."""
    NULL = 0
    BOOL = 1
    INT = 2
    FLOAT = 3
    STRING = 4
    LIST = 5
    OBJECT = 6
    DATE = 7
    REFERENCE = 8


@dataclass
class IR_Instruction:
    """A single IR instruction."""
    opcode: Opcode
    operand: Any = None
    
    def __repr__(self) -> str:
        if self.operand is not None:
            return f"{self.opcode.name} {self.operand}"
        return self.opcode.name


# =============================================================================
# Compiled Expression
# =============================================================================

@dataclass
class IR_Expr:
    """Compiled expression as IR bytecode."""
    uid: str
    instructions: List[IR_Instruction] = field(default_factory=list)
    result_type: IR_Type = IR_Type.NULL
    pure: bool = False
    
    source_expr_uid: Optional[str] = None
    compiled_at: datetime = field(default_factory=datetime.utcnow)
    constants: List[Any] = field(default_factory=list)
    
    def add(self, opcode: Opcode, operand: Any = None) -> "IR_Expr":
        """Add an instruction (fluent interface)."""
        self.instructions.append(IR_Instruction(opcode, operand))
        return self
    
    def __len__(self) -> int:
        return len(self.instructions)


# =============================================================================
# Compiled Step
# =============================================================================

@dataclass
class IR_Step:
    """
    Compiled workflow step.
    
    Replaces IR_Element for execution context.
    """
    uid: str
    type: str
    
    # Static params (constants)
    static_params: Dict[str, Any] = field(default_factory=dict)
    
    # Dynamic params (expression UIDs)
    dynamic_params: Dict[str, str] = field(default_factory=dict)
    
    # Compiled expressions for this step
    expressions: Dict[str, IR_Expr] = field(default_factory=dict)
    
    # Dependencies (step UIDs)
    depends_on: List[str] = field(default_factory=list)
    
    # Condition expression
    condition_expr: Optional[str] = None
    
    # Error handling
    on_error: str = "stop"
    
    # Timeout (ms)
    timeout_ms: int = 30000


# =============================================================================
# Compiled Workflow
# =============================================================================

@dataclass
class IR_Workflow:
    """
    Compiled workflow - the final artifact for execution.
    
    Replaces IR_App with execution-only concerns.
    No pages, no elements, no UI.
    """
    uid: str
    version: int
    
    # All steps (keyed by UID)
    steps: Dict[str, IR_Step] = field(default_factory=dict)
    
    # Execution order (step UIDs)
    execution_order: List[str] = field(default_factory=list)
    
    # All expressions
    expressions: Dict[str, IR_Expr] = field(default_factory=dict)
    
    # Initial variables
    variables: Dict[str, Any] = field(default_factory=dict)
    
    # Dependency graph
    dependency_graph: Dict[str, Set[str]] = field(default_factory=dict)
    
    # Compilation metadata
    compiled_at: datetime = field(default_factory=datetime.utcnow)
    compiler_version: str = "1.0.0"
    
    # Integrity
    checksum: Optional[str] = None
    
    @property
    def stats(self) -> Dict[str, int]:
        """Get compilation statistics."""
        return {
            "steps": len(self.steps),
            "expressions": len(self.expressions),
            "variables": len(self.variables),
        }
