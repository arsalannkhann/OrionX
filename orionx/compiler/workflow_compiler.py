"""
OneX Workflow Compiler

Compiles Workflow into ExecutionPlan.
Copied from OrionX with minimal changes.
"""

from __future__ import annotations
from typing import Dict, List, Set, Optional, Tuple
from collections import deque
from datetime import datetime
import re

from .execution_plan import (
    ExecutionPlan,
    ExecutionGroup,
    ValidationResult,
    ValidationIssue,
    ValidationSeverity,
    CompilationError,
)


# UID validation patterns
UID_PATTERNS = {
    "workflow": re.compile(r"^wf_[a-z0-9]{3,20}$"),
    "node": re.compile(r"^(step|node)_[a-z0-9]{3,12}$"),
    "edge": re.compile(r"^edge_[a-z0-9]{3,12}$"),
    "trigger": re.compile(r"^trigger$"),
}

# Required config fields by step type
REQUIRED_STEP_CONFIG = {
    "api_call": ["method", "url"],
    "create_entity": ["entity_type"],
    "update_entity": ["entity_uid"],
    "delete_entity": ["entity_uid"],
    "send_email": ["to"],
    "condition": ["expression"],
    "loop": ["collection"],
}


class WorkflowCompiler:
    """
    Compiles IR_Workflow JSON into ExecutionPlan.
    
    Features:
    - Validates DAG structure
    - Detects cycles
    - Topological sort
    - Parallel group computation
    """
    
    def __init__(self):
        self.version = "1.0.0"
    
    def validate(self, ir_workflow: Dict) -> ValidationResult:
        """Validate an IR_Workflow without compiling."""
        errors = []
        warnings = []
        
        # Validate required fields
        if "uid" not in ir_workflow:
            errors.append(ValidationIssue(
                code="E_MISSING_UID",
                severity=ValidationSeverity.BLOCKING,
                message="Workflow missing required 'uid' field"
            ))
        
        if "version" not in ir_workflow:
            errors.append(ValidationIssue(
                code="E_MISSING_VERSION",
                severity=ValidationSeverity.BLOCKING,
                message="Workflow missing required 'version' field"
            ))
        
        # Build node/edge maps
        nodes = ir_workflow.get("nodes", [])
        edges = ir_workflow.get("edges", [])
        
        node_uids: Set[str] = {"trigger"}
        for node in nodes:
            uid = node.get("uid", "")
            if uid in node_uids:
                errors.append(ValidationIssue(
                    code="E_DUPLICATE_UID",
                    severity=ValidationSeverity.BLOCKING,
                    message=f"Duplicate node UID: {uid}",
                    nodes=[uid]
                ))
            node_uids.add(uid)
        
        # Validate edges
        for edge in edges:
            source = edge.get("source", "")
            target = edge.get("target", "")
            edge_uid = edge.get("uid", "unknown")
            
            if source not in node_uids:
                errors.append(ValidationIssue(
                    code="E_MISSING_REF",
                    severity=ValidationSeverity.BLOCKING,
                    message=f"Edge references non-existent source: {source}",
                    edges=[edge_uid]
                ))
            
            if target not in node_uids:
                errors.append(ValidationIssue(
                    code="E_MISSING_REF",
                    severity=ValidationSeverity.BLOCKING,
                    message=f"Edge references non-existent target: {target}",
                    edges=[edge_uid]
                ))
        
        # Detect cycles
        cycle = self._detect_cycle(nodes, edges)
        if cycle:
            errors.append(ValidationIssue(
                code="E_CYCLE",
                severity=ValidationSeverity.BLOCKING,
                message=f"Cycle detected: {' -> '.join(cycle)}",
                nodes=cycle
            ))
        
        # Check for orphan nodes
        reachable = self._find_reachable("trigger", edges)
        for node in nodes:
            uid = node.get("uid", "")
            if uid not in reachable:
                warnings.append(ValidationIssue(
                    code="W_ORPHAN",
                    severity=ValidationSeverity.WARNING,
                    message=f"Node {uid} is unreachable from trigger",
                    nodes=[uid]
                ))
        
        valid = len(errors) == 0
        return ValidationResult(valid=valid, errors=errors, warnings=warnings)
    
    def compile(self, ir_workflow: Dict) -> ExecutionPlan:
        """Compile an IR_Workflow into an ExecutionPlan."""
        # Validate
        validation = self.validate(ir_workflow)
        if validation.has_blocking():
            first_error = validation.errors[0]
            raise CompilationError(
                code=first_error.code,
                message=first_error.message,
                nodes=first_error.nodes,
                edges=first_error.edges
            )
        
        # Extract data
        nodes = ir_workflow.get("nodes", [])
        edges = ir_workflow.get("edges", [])
        
        # Build adjacency lists
        adjacency: Dict[str, List[str]] = {"trigger": []}
        in_degree: Dict[str, int] = {"trigger": 0}
        
        for node in nodes:
            uid = node.get("uid")
            adjacency[uid] = []
            in_degree[uid] = 0
        
        for edge in edges:
            source = edge.get("source")
            target = edge.get("target")
            adjacency[source].append(target)
            in_degree[target] += 1
        
        # Topological sort with depth tracking
        sorted_nodes, node_depths = self._topological_sort_with_depth(adjacency, in_degree)
        
        # Group by depth for parallel execution
        max_depth = max(node_depths.values()) if node_depths else 0
        groups = []
        for depth in range(max_depth + 1):
            nodes_at_depth = [uid for uid, d in node_depths.items() if d == depth]
            nodes_at_depth.sort()
            groups.append(ExecutionGroup(depth=depth, node_uids=nodes_at_depth))
        
        return ExecutionPlan(
            workflow_uid=ir_workflow.get("uid", ""),
            version=ir_workflow.get("version", 1),
            groups=groups,
            variable_bindings=ir_workflow.get("variables", {}),
            validation=validation,
            compiled_at=datetime.utcnow().isoformat(),
            compiler_version=self.version
        )
    
    def _detect_cycle(self, nodes: List[Dict], edges: List[Dict]) -> Optional[List[str]]:
        """Detect cycles using DFS."""
        all_uids = {"trigger"} | {n.get("uid") for n in nodes}
        adjacency: Dict[str, List[str]] = {uid: [] for uid in all_uids}
        
        for edge in edges:
            source = edge.get("source")
            target = edge.get("target")
            if source in adjacency:
                adjacency[source].append(target)
        
        WHITE, GRAY, BLACK = 0, 1, 2
        color = {uid: WHITE for uid in all_uids}
        parent = {uid: None for uid in all_uids}
        
        def dfs(node: str) -> Optional[List[str]]:
            color[node] = GRAY
            for neighbor in adjacency.get(node, []):
                if color.get(neighbor) == GRAY:
                    cycle = [neighbor, node]
                    current = node
                    while parent[current] != neighbor and parent[current] is not None:
                        current = parent[current]
                        cycle.append(current)
                    return cycle
                elif color.get(neighbor) == WHITE:
                    parent[neighbor] = node
                    result = dfs(neighbor)
                    if result:
                        return result
            color[node] = BLACK
            return None
        
        for uid in all_uids:
            if color[uid] == WHITE:
                result = dfs(uid)
                if result:
                    return result
        
        return None
    
    def _find_reachable(self, start: str, edges: List[Dict]) -> Set[str]:
        """Find all nodes reachable from start using BFS."""
        adjacency: Dict[str, List[str]] = {}
        for edge in edges:
            source = edge.get("source")
            target = edge.get("target")
            if source not in adjacency:
                adjacency[source] = []
            adjacency[source].append(target)
        
        visited = {start}
        queue = deque([start])
        
        while queue:
            node = queue.popleft()
            for neighbor in adjacency.get(node, []):
                if neighbor not in visited:
                    visited.add(neighbor)
                    queue.append(neighbor)
        
        return visited
    
    def _topological_sort_with_depth(
        self,
        adjacency: Dict[str, List[str]],
        in_degree: Dict[str, int]
    ) -> Tuple[List[str], Dict[str, int]]:
        """Topological sort using Kahn's algorithm."""
        queue = deque([uid for uid, deg in in_degree.items() if deg == 0])
        result = []
        depths = {uid: 0 for uid in queue}
        
        while queue:
            queue = deque(sorted(queue))
            node = queue.popleft()
            result.append(node)
            
            for neighbor in adjacency.get(node, []):
                in_degree[neighbor] -= 1
                depths[neighbor] = max(depths.get(neighbor, 0), depths[node] + 1)
                if in_degree[neighbor] == 0:
                    queue.append(neighbor)
        
        return result, depths


# Singleton instance
workflow_compiler = WorkflowCompiler()
