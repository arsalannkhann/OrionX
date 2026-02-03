
import pytest
from orionx.compiler.workflow_compiler import workflow_compiler

def test_compiler_handles_trigger_node():
    """
    Verify that the optimized compiler correctly handles the 'trigger' node
    and doesn't flag reachable nodes as orphans.
    """
    workflow = {
        "uid": "wf_test_trigger",
        "version": 1,
        "nodes": [
            {"uid": "step1"},
            {"uid": "step2"}
        ],
        "edges": [
            {"source": "trigger", "target": "step1"},
            {"source": "step1", "target": "step2"}
        ]
    }

    result = workflow_compiler.validate(workflow)

    assert result.valid is True
    assert len(result.errors) == 0
    # If trigger is ignored, step1 and step2 would be orphans
    assert len(result.warnings) == 0, f"Found warnings: {result.warnings}"

def test_compiler_detects_orphans():
    """
    Verify that actual orphans are still detected.
    """
    workflow = {
        "uid": "wf_test_orphan",
        "version": 1,
        "nodes": [
            {"uid": "step1"},
            {"uid": "orphan_step"}
        ],
        "edges": [
            {"source": "trigger", "target": "step1"}
            # orphan_step is not connected
        ]
    }

    result = workflow_compiler.validate(workflow)

    assert result.valid is True # Orphans are warnings, not errors
    orphan_warnings = [w for w in result.warnings if w.code == "W_ORPHAN"]
    assert len(orphan_warnings) == 1
    assert "orphan_step" in orphan_warnings[0].message
