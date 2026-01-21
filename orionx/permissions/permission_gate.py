"""
OneX Permission Gate

Enforces data access rules at runtime.
Supports row-level security and field-level access control.
Copied from OrionX with EvaluationContext → ExecutionContext.
"""

from __future__ import annotations
from typing import Dict, List, Any, Optional, Set
from dataclasses import dataclass
from enum import Enum
import logging

from ..schemas.data_types import (
    DataType,
    PrivacyRule,
    PrivacyLevel,
    PrivacyAction,
    DataField,
)
from ..schemas.execution import ExecutionContext


logger = logging.getLogger(__name__)


# =============================================================================
# Permission Results
# =============================================================================

class PermissionDecision(str, Enum):
    """Permission decision types."""
    ALLOW = "allow"
    DENY = "deny"
    FILTER = "filter"


@dataclass
class PermissionResult:
    """Result of a permission check."""
    decision: PermissionDecision
    allowed_fields: Optional[Set[str]] = None
    denied_reason: Optional[str] = None
    rule_uid: Optional[str] = None
    
    @property
    def is_allowed(self) -> bool:
        return self.decision in (PermissionDecision.ALLOW, PermissionDecision.FILTER)
    
    @classmethod
    def allow(cls) -> "PermissionResult":
        return cls(decision=PermissionDecision.ALLOW)
    
    @classmethod
    def deny(cls, reason: str = "Access denied") -> "PermissionResult":
        return cls(decision=PermissionDecision.DENY, denied_reason=reason)
    
    @classmethod
    def filter(cls, allowed_fields: Set[str]) -> "PermissionResult":
        return cls(decision=PermissionDecision.FILTER, allowed_fields=allowed_fields)


# =============================================================================
# Permission Gate
# =============================================================================

class PermissionGate:
    """
    Enforces data access rules at runtime.
    
    Features:
    - Row-level security
    - Field-level access control
    - Action-based permissions (view, create, update, delete)
    - Expression-based custom rules
    """
    
    def __init__(
        self,
        expression_evaluator=None,
        expressions: Optional[Dict[str, Any]] = None
    ):
        self.evaluator = expression_evaluator
        self.expressions = expressions or {}
    
    def check(
        self,
        data_type: DataType,
        action: PrivacyAction,
        entity: Optional[Dict[str, Any]],
        context: ExecutionContext
    ) -> PermissionResult:
        """
        Check if action is allowed on an entity.
        
        Args:
            data_type: The data type being accessed
            action: The action being performed
            entity: The specific record (None for create)
            context: Current execution context
            
        Returns:
            PermissionResult indicating decision
        """
        rule = data_type.get_rule(action)
        
        if rule is None:
            logger.warning(f"No privacy rule for {action} on {data_type.uid}")
            return PermissionResult.deny(f"No {action} rule defined")
        
        return self._evaluate_rule(rule, entity, context)
    
    def _evaluate_rule(
        self,
        rule: PrivacyRule,
        entity: Optional[Dict[str, Any]],
        context: ExecutionContext
    ) -> PermissionResult:
        """Evaluate a privacy rule."""
        level = rule.level
        
        if level == PrivacyLevel.PUBLIC:
            return PermissionResult.allow()
        
        if level in (PrivacyLevel.PRIVATE, PrivacyLevel.CREATOR_ONLY):
            return self._check_creator(entity, context, rule.uid)
        
        if level == PrivacyLevel.LOGGED_IN:
            if context.user is not None:
                return PermissionResult.allow()
            return PermissionResult.deny("Authentication required")
        
        if level == PrivacyLevel.CUSTOM:
            return self._evaluate_custom_rule(rule, entity, context)
        
        return PermissionResult.deny("Unknown privacy level")
    
    def _check_creator(
        self,
        entity: Optional[Dict[str, Any]],
        context: ExecutionContext,
        rule_uid: str
    ) -> PermissionResult:
        """Check if current user is the creator."""
        if context.user is None:
            return PermissionResult.deny("Authentication required")
        
        if entity is None:
            return PermissionResult.allow()
        
        created_by = entity.get("created_by") or entity.get("creator_id") or entity.get("owner_id")
        user_id = context.user.get("uid") or context.user.get("id")
        
        if created_by == user_id:
            return PermissionResult(
                decision=PermissionDecision.ALLOW,
                rule_uid=rule_uid
            )
        
        return PermissionResult.deny("Only the creator can access this record")
    
    def _evaluate_custom_rule(
        self,
        rule: PrivacyRule,
        entity: Optional[Dict[str, Any]],
        context: ExecutionContext
    ) -> PermissionResult:
        """Evaluate a custom expression-based rule."""
        if not rule.condition_expr:
            return PermissionResult.deny("Custom rule has no condition")
        
        if not self.evaluator:
            return PermissionResult.deny("No expression evaluator configured")
        
        expression = self.expressions.get(rule.condition_expr)
        if not expression:
            logger.error(f"Expression {rule.condition_expr} not found")
            return PermissionResult.deny("Rule expression not found")
        
        eval_context = context.with_entity(entity, "record") if entity else context
        
        try:
            result = self.evaluator.evaluate(expression, eval_context)
            
            if result:
                return PermissionResult(
                    decision=PermissionDecision.ALLOW,
                    rule_uid=rule.uid
                )
            else:
                return PermissionResult.deny("Custom rule condition not met")
                
        except Exception as e:
            logger.error(f"Error evaluating rule {rule.uid}: {e}")
            return PermissionResult.deny(f"Rule evaluation error: {e}")
    
    # =========================================================================
    # Field-Level Permissions
    # =========================================================================
    
    def filter_fields(
        self,
        data_type: DataType,
        entity: Dict[str, Any],
        context: ExecutionContext
    ) -> Dict[str, Any]:
        """Filter out fields the user cannot access."""
        result = {}
        
        for field in data_type.fields:
            if self._can_access_field(field, entity, context):
                if field.name in entity:
                    result[field.name] = entity[field.name]
        
        # Always include system fields
        for sys_field in ["id", "uid", "created_at", "updated_at"]:
            if sys_field in entity:
                result[sys_field] = entity[sys_field]
        
        return result
    
    def _can_access_field(
        self,
        field: DataField,
        entity: Dict[str, Any],
        context: ExecutionContext
    ) -> bool:
        """Check if user can access a specific field."""
        if field.privacy is None:
            return True
        
        if field.privacy == PrivacyLevel.PUBLIC:
            return True
        
        if field.privacy == PrivacyLevel.LOGGED_IN:
            return context.user is not None
        
        if field.privacy == PrivacyLevel.CREATOR_ONLY:
            if context.user is None:
                return False
            created_by = entity.get("created_by") or entity.get("creator_id")
            user_id = context.user.get("uid") or context.user.get("id")
            return created_by == user_id
        
        return True
    
    # =========================================================================
    # Batch Operations
    # =========================================================================
    
    def filter_list(
        self,
        data_type: DataType,
        entities: List[Dict[str, Any]],
        action: PrivacyAction,
        context: ExecutionContext
    ) -> List[Dict[str, Any]]:
        """Filter a list to only include accessible records."""
        result = []
        
        for entity in entities:
            permission = self.check(data_type, action, entity, context)
            if permission.is_allowed:
                filtered = self.filter_fields(data_type, entity, context)
                result.append(filtered)
        
        return result
