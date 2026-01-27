"""
OneX State Store

Manages all runtime state with signal-based reactivity.
Refactored from OrionX to use execution-oriented scopes.

Key changes:
- StateScope.PAGE → StateScope.STEP (no page concept)
- StateScope.SESSION → StateScope.WORKFLOW (workflow-scoped)
- StateScope.APP → StateScope.GLOBAL (persistent global)
- StateScope.URL removed (no URL concept)
"""

from __future__ import annotations
from typing import Dict, List, Any, Callable, Set
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime
import logging


logger = logging.getLogger(__name__)


# =============================================================================
# State Scopes
# =============================================================================

class StateScope(str, Enum):
    """
    Scope of state storage.
    
    Execution-oriented (no UI concepts):
    - STEP: Cleared after step completes
    - WORKFLOW: Cleared after workflow completes
    - GLOBAL: Persisted across workflows
    - USER: Persisted per user in database
    """
    STEP = "step"           # Cleared after step
    WORKFLOW = "workflow"   # Cleared after workflow
    GLOBAL = "global"       # Persisted globally
    USER = "user"           # Persisted per user


# =============================================================================
# Signal System
# =============================================================================

@dataclass
class Signal:
    """A reactive signal that notifies subscribers on change."""
    key: str
    value: Any
    scope: StateScope
    subscribers: Set[str] = field(default_factory=set)
    updated_at: datetime = field(default_factory=datetime.utcnow)
    
    def set(self, new_value: Any) -> bool:
        """Set value and return True if changed."""
        if self.value != new_value:
            self.value = new_value
            self.updated_at = datetime.utcnow()
            return True
        return False


# =============================================================================
# State Store
# =============================================================================

class StateStore:
    """
    Central state management for workflow execution.
    
    Features:
    - Multi-scope state (step, workflow, global, user)
    - Signal-based subscriptions
    - Batch updates
    - State persistence
    """
    
    def __init__(self, persistence_backend=None):
        """
        Initialize state store.
        
        Args:
            persistence_backend: Optional backend for user-scope persistence
        """
        self._signals: Dict[str, Signal] = {}
        self._subscribers: Dict[str, List[Callable[[Any], None]]] = {}
        self._batch_mode = False
        self._batch_changes: Set[str] = set()
        self._persistence = persistence_backend
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get a state value."""
        if key in self._signals:
            return self._signals[key].value
        return default
    
    def set(
        self,
        key: str,
        value: Any,
        scope: StateScope = StateScope.WORKFLOW
    ) -> None:
        """
        Set a state value.
        
        Args:
            key: State key
            value: New value
            scope: State scope for lifecycle/persistence
        """
        if key not in self._signals:
            self._signals[key] = Signal(key=key, value=value, scope=scope)
            changed = True
        else:
            signal = self._signals[key]
            changed = signal.set(value)
        
        if changed:
            if self._batch_mode:
                self._batch_changes.add(key)
            else:
                self._notify(key)
    
    def delete(self, key: str) -> None:
        """Delete a state value."""
        if key in self._signals:
            del self._signals[key]
            self._notify(key, deleted=True)
    
    def subscribe(
        self,
        key: str,
        callback: Callable[[Any], None]
    ) -> Callable[[], None]:
        """
        Subscribe to state changes.
        
        Args:
            key: State key to watch
            callback: Function called with new value on change
            
        Returns:
            Unsubscribe function
        """
        if key not in self._subscribers:
            self._subscribers[key] = []
        
        self._subscribers[key].append(callback)
        
        def unsubscribe():
            if key in self._subscribers:
                self._subscribers[key].remove(callback)
        
        return unsubscribe
    
    def _notify(self, key: str, deleted: bool = False) -> None:
        """Notify subscribers of a change."""
        if key in self._subscribers:
            value = None if deleted else self.get(key)
            for callback in self._subscribers[key]:
                try:
                    callback(value)
                except Exception as e:
                    logger.error(f"Subscriber error for {key}: {e}")
    
    # =========================================================================
    # Batch Operations
    # =========================================================================
    
    def batch_start(self) -> None:
        """Start a batch update (defer notifications)."""
        self._batch_mode = True
        self._batch_changes.clear()
    
    def batch_commit(self) -> None:
        """Commit batch and notify all changed keys."""
        self._batch_mode = False
        for key in self._batch_changes:
            self._notify(key)
        self._batch_changes.clear()
    
    def batch_rollback(self) -> None:
        """Cancel batch update."""
        self._batch_mode = False
        self._batch_changes.clear()
    
    # =========================================================================
    # Scope Operations
    # =========================================================================
    
    def get_scope(self, scope: StateScope) -> Dict[str, Any]:
        """Get all values in a scope."""
        return {
            key: signal.value
            for key, signal in self._signals.items()
            if signal.scope == scope
        }
    
    def clear_scope(self, scope: StateScope) -> None:
        """Clear all values in a scope."""
        keys_to_delete = [
            key for key, signal in self._signals.items()
            if signal.scope == scope
        ]
        for key in keys_to_delete:
            self.delete(key)
    
    # =========================================================================
    # Persistence
    # =========================================================================
    
    async def persist_user_state(self, user_id: str) -> None:
        """Persist user-scope state to backend."""
        if not self._persistence:
            return
        
        user_state = self.get_scope(StateScope.USER)
        await self._persistence.save_user_state(user_id, user_state)
    
    async def load_user_state(self, user_id: str) -> None:
        """Load user-scope state from backend."""
        if not self._persistence:
            return
        
        user_state = await self._persistence.load_user_state(user_id)
        if user_state:
            for key, value in user_state.items():
                self.set(key, value, StateScope.USER)
    
    def to_dict(self) -> Dict[str, Any]:
        """Serialize all state to dictionary."""
        result: Dict[str, Dict[str, Any]] = {}
        for scope in StateScope:
            result[scope.value] = self.get_scope(scope)
        return result
    
    def from_dict(self, data: Dict[str, Any]) -> None:
        """Load state from dictionary."""
        for scope_name, values in data.items():
            try:
                scope = StateScope(scope_name)
                for key, value in values.items():
                    self.set(key, value, scope)
            except ValueError:
                logger.warning(f"Unknown scope: {scope_name}")
    
    # =========================================================================
    # Computed Values
    # =========================================================================
    
    def computed(
        self,
        key: str,
        dependencies: List[str],
        compute_fn: Callable[[], Any]
    ) -> None:
        """
        Create a computed value that updates when dependencies change.
        
        Args:
            key: Key for the computed value
            dependencies: Keys this value depends on
            compute_fn: Function to compute the value
        """
        def recompute(_=None):
            value = compute_fn()
            self.set(key, value, StateScope.WORKFLOW)
        
        for dep in dependencies:
            self.subscribe(dep, recompute)
        
        # Initial computation
        recompute()
    
    # =========================================================================
    # Workflow Lifecycle
    # =========================================================================
    
    def on_workflow_start(self) -> None:
        """Called when a workflow starts."""
        # Clear step-scoped state from previous workflows
        self.clear_scope(StateScope.STEP)
    
    def on_workflow_end(self) -> None:
        """Called when a workflow ends."""
        self.clear_scope(StateScope.STEP)
        self.clear_scope(StateScope.WORKFLOW)
    
    def on_step_start(self) -> None:
        """Called when a step starts."""
        # Step state is preserved within workflow
        pass
    
    def on_step_end(self) -> None:
        """Called when a step ends."""
        self.clear_scope(StateScope.STEP)
