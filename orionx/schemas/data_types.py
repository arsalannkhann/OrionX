"""
OneX Data Type Schema

Data type definitions with privacy rules.
Copied from OrionX with minimal changes.
"""

from __future__ import annotations
from typing import Dict, List, Any, Optional
from pydantic import BaseModel, Field, field_validator
from enum import Enum
from uuid import uuid4


# =============================================================================
# Field Types
# =============================================================================

class FieldType(str, Enum):
    """Supported field types."""
    TEXT = "text"
    NUMBER = "number"
    BOOLEAN = "boolean"
    DATE = "date"
    DATETIME = "datetime"
    IMAGE = "image"
    FILE = "file"
    REFERENCE = "reference"
    LIST = "list"
    OBJECT = "object"
    GEOLOCATION = "geolocation"


# =============================================================================
# Privacy Levels
# =============================================================================

class PrivacyLevel(str, Enum):
    """Built-in privacy shortcuts."""
    PUBLIC = "public"
    PRIVATE = "private"
    CREATOR_ONLY = "creator_only"
    LOGGED_IN = "logged_in"
    CUSTOM = "custom"


class PrivacyAction(str, Enum):
    """Actions that can be controlled by privacy rules."""
    VIEW = "view"
    CREATE = "create"
    UPDATE = "update"
    DELETE = "delete"


# =============================================================================
# Field Schema
# =============================================================================

class DataField(BaseModel):
    """Definition of a field within a data type."""
    uid: str = Field(default_factory=lambda: f"field_{uuid4().hex[:8]}")
    name: str = Field(..., min_length=1, max_length=100)
    type: FieldType
    
    # Constraints
    required: bool = False
    unique: bool = False
    indexed: bool = False
    
    # Default value
    default: Optional[Any] = None
    
    # For reference fields
    reference_type: Optional[str] = None
    
    # For list fields
    list_type: Optional[FieldType] = None
    
    # Validation rules
    validation: Dict[str, Any] = Field(default_factory=dict)
    
    # Field-level privacy
    privacy: Optional[PrivacyLevel] = None
    
    @field_validator("uid")
    @classmethod
    def validate_uid(cls, v: str) -> str:
        if not v.startswith("field_"):
            raise ValueError("Field UID must start with 'field_'")
        return v


# =============================================================================
# Privacy Rule Schema
# =============================================================================

class PrivacyRule(BaseModel):
    """Row-level security rule for a data type."""
    uid: str = Field(default_factory=lambda: f"rule_{uuid4().hex[:8]}")
    action: PrivacyAction
    level: PrivacyLevel = PrivacyLevel.CUSTOM
    
    # For custom rules - expression
    condition_expr: Optional[str] = None
    
    # Description
    description: Optional[str] = None
    
    @field_validator("uid")
    @classmethod
    def validate_uid(cls, v: str) -> str:
        if not v.startswith("rule_"):
            raise ValueError("Rule UID must start with 'rule_'")
        return v


# =============================================================================
# Main Data Type Schema
# =============================================================================

class DataType(BaseModel):
    """
    OneX Data Type Schema.
    
    Represents a data entity type with field definitions and privacy rules.
    """
    uid: str = Field(default_factory=lambda: f"type_{uuid4().hex[:8]}")
    name: str = Field(..., min_length=1, max_length=100)
    plural_name: Optional[str] = None
    description: Optional[str] = Field(default=None, max_length=1000)
    
    # Fields
    fields: List[DataField] = Field(default_factory=list)
    
    # Privacy rules
    privacy_rules: List[PrivacyRule] = Field(default_factory=list)
    
    # System fields
    has_created_at: bool = True
    has_updated_at: bool = True
    has_created_by: bool = True
    
    # Soft delete
    soft_delete: bool = False
    
    @field_validator("uid")
    @classmethod
    def validate_uid(cls, v: str) -> str:
        if not v.startswith("type_"):
            raise ValueError("DataType UID must start with 'type_'")
        return v
    
    def get_field(self, name: str) -> Optional[DataField]:
        """Get a field by name."""
        for field in self.fields:
            if field.name == name:
                return field
        return None
    
    def get_rule(self, action: PrivacyAction) -> Optional[PrivacyRule]:
        """Get the privacy rule for a specific action."""
        for rule in self.privacy_rules:
            if rule.action == action:
                return rule
        return None
