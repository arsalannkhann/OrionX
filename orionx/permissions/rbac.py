"""
OneX RBAC (Role-Based Access Control)

Simple RBAC implementation for workflow execution permissions.
"""

from __future__ import annotations
from typing import Dict, List, Set, Optional
from dataclasses import dataclass, field
from enum import Enum
import logging


logger = logging.getLogger(__name__)


# =============================================================================
# Permission Types
# =============================================================================

class Permission(str, Enum):
    """System-level permissions."""
    # Workflow permissions
    WORKFLOW_EXECUTE = "workflow:execute"
    WORKFLOW_CANCEL = "workflow:cancel"
    WORKFLOW_VIEW = "workflow:view"
    WORKFLOW_CREATE = "workflow:create"
    WORKFLOW_UPDATE = "workflow:update"
    WORKFLOW_DELETE = "workflow:delete"
    
    # Entity permissions
    ENTITY_CREATE = "entity:create"
    ENTITY_READ = "entity:read"
    ENTITY_UPDATE = "entity:update"
    ENTITY_DELETE = "entity:delete"
    
    # Plugin permissions
    PLUGIN_INSTALL = "plugin:install"
    PLUGIN_CONFIGURE = "plugin:configure"
    
    # Admin permissions
    ADMIN_USERS = "admin:users"
    ADMIN_SETTINGS = "admin:settings"
    ADMIN_AUDIT = "admin:audit"


# =============================================================================
# Role Definition
# =============================================================================

@dataclass
class Role:
    """A role with a set of permissions."""
    uid: str
    name: str
    description: Optional[str] = None
    permissions: Set[Permission] = field(default_factory=set)
    is_system: bool = False  # System roles cannot be modified
    
    def has_permission(self, permission: Permission) -> bool:
        """Check if role has a permission."""
        return permission in self.permissions
    
    def grant(self, permission: Permission) -> None:
        """Grant a permission to this role."""
        if self.is_system:
            raise ValueError("Cannot modify system role")
        self.permissions.add(permission)
    
    def revoke(self, permission: Permission) -> None:
        """Revoke a permission from this role."""
        if self.is_system:
            raise ValueError("Cannot modify system role")
        self.permissions.discard(permission)


# =============================================================================
# Built-in Roles
# =============================================================================

ROLE_ADMIN = Role(
    uid="role_admin",
    name="Administrator",
    description="Full system access",
    permissions=set(Permission),  # All permissions
    is_system=True,
)

ROLE_OPERATOR = Role(
    uid="role_operator",
    name="Operator",
    description="Can execute and manage workflows",
    permissions={
        Permission.WORKFLOW_EXECUTE,
        Permission.WORKFLOW_CANCEL,
        Permission.WORKFLOW_VIEW,
        Permission.ENTITY_CREATE,
        Permission.ENTITY_READ,
        Permission.ENTITY_UPDATE,
    },
    is_system=True,
)

ROLE_VIEWER = Role(
    uid="role_viewer",
    name="Viewer",
    description="Read-only access",
    permissions={
        Permission.WORKFLOW_VIEW,
        Permission.ENTITY_READ,
    },
    is_system=True,
)


# =============================================================================
# RBAC Manager
# =============================================================================

class RBACManager:
    """
    Manages roles and user role assignments.
    
    Usage:
        rbac = RBACManager()
        rbac.assign_role(user_uid, "role_operator")
        if rbac.check_permission(user_uid, Permission.WORKFLOW_EXECUTE):
            # Allow execution
    """
    
    def __init__(self):
        # Built-in roles
        self._roles: Dict[str, Role] = {
            ROLE_ADMIN.uid: ROLE_ADMIN,
            ROLE_OPERATOR.uid: ROLE_OPERATOR,
            ROLE_VIEWER.uid: ROLE_VIEWER,
        }
        
        # User -> Role assignments
        self._user_roles: Dict[str, Set[str]] = {}
    
    def get_role(self, role_uid: str) -> Optional[Role]:
        """Get a role by UID."""
        return self._roles.get(role_uid)
    
    def create_role(
        self,
        uid: str,
        name: str,
        permissions: Set[Permission],
        description: Optional[str] = None,
    ) -> Role:
        """Create a new custom role."""
        if uid in self._roles:
            raise ValueError(f"Role already exists: {uid}")
        
        role = Role(
            uid=uid,
            name=name,
            description=description,
            permissions=permissions,
            is_system=False,
        )
        self._roles[uid] = role
        return role
    
    def delete_role(self, role_uid: str) -> bool:
        """Delete a custom role."""
        role = self._roles.get(role_uid)
        if not role:
            return False
        if role.is_system:
            raise ValueError("Cannot delete system role")
        
        del self._roles[role_uid]
        
        # Remove from all user assignments
        for roles in self._user_roles.values():
            roles.discard(role_uid)
        
        return True
    
    def assign_role(self, user_uid: str, role_uid: str) -> None:
        """Assign a role to a user."""
        if role_uid not in self._roles:
            raise ValueError(f"Role not found: {role_uid}")
        
        if user_uid not in self._user_roles:
            self._user_roles[user_uid] = set()
        
        self._user_roles[user_uid].add(role_uid)
        logger.info(f"Assigned role {role_uid} to user {user_uid}")
    
    def revoke_role(self, user_uid: str, role_uid: str) -> None:
        """Revoke a role from a user."""
        if user_uid in self._user_roles:
            self._user_roles[user_uid].discard(role_uid)
            logger.info(f"Revoked role {role_uid} from user {user_uid}")
    
    def get_user_roles(self, user_uid: str) -> List[Role]:
        """Get all roles assigned to a user."""
        role_uids = self._user_roles.get(user_uid, set())
        return [self._roles[uid] for uid in role_uids if uid in self._roles]
    
    def get_user_permissions(self, user_uid: str) -> Set[Permission]:
        """Get all permissions for a user (union of all role permissions)."""
        permissions: Set[Permission] = set()
        for role in self.get_user_roles(user_uid):
            permissions.update(role.permissions)
        return permissions
    
    def check_permission(self, user_uid: str, permission: Permission) -> bool:
        """Check if a user has a specific permission."""
        return permission in self.get_user_permissions(user_uid)
    
    def require_permission(self, user_uid: str, permission: Permission) -> None:
        """Require a permission, raising PermissionError if not met."""
        if not self.check_permission(user_uid, permission):
            raise PermissionError(f"User {user_uid} lacks permission: {permission.value}")
