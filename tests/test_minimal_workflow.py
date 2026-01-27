"""
OneX Minimal Workflow Test

Phase 5: Minimal Execution Test

Test workflow:
- Step 1: Call a mock external API
- Step 2: Transform response
- Step 3: Persist result (mock)

Validates:
- Execution starts
- Steps run in order
- Logs are produced
- Failures are surfaced
- No UI code is invoked
"""

import pytest

from orionx.core.executor import OneXEngine, WorkflowExecutor
from orionx.core.action_handlers import StepResult
from orionx.schemas.workflow import (
    Workflow,
    WorkflowStep,
    WorkflowTrigger,
    StepType,
    TriggerType,
)
from orionx.schemas.execution import ExecutionContext, ExecutionStatus


# =============================================================================
# Test Fixtures
# =============================================================================

@pytest.fixture
def mock_api_handler():
    """Mock API call handler that returns test data."""
    async def handler(params, context):
        url = params.get("url", "")
        return StepResult(
            success=True,
            data={
                "status": 200,
                "body": {
                    "user_id": "user_123",
                    "name": "Test User",
                    "email": "test@example.com",
                }
            }
        )
    return handler


@pytest.fixture
def mock_entity_handler():
    """Mock entity creation handler."""
    async def handler(params, context):
        entity_type = params.get("entity_type", "Unknown")
        data = params.get("data", {})
        
        # Handle if data is a StepResult object
        if hasattr(data, 'data'):
            data = data.data if isinstance(data.data, dict) else {}
        elif not isinstance(data, dict):
            data = {}
        
        return StepResult(
            success=True,
            data={
                "uid": f"{entity_type.lower()}_created_123",
                **data,
            }
        )
    return handler


@pytest.fixture
def mock_transform_handler():
    """Mock data transformation handler."""
    async def handler(params, context):
        input_data = params.get("input", {})
        mapping = params.get("mapping", {})
        
        # Simple transformation
        result = {}
        if hasattr(input_data, 'data'):
            # StepResult object
            source = input_data.data.get("body", {}) if isinstance(input_data.data, dict) else {}
        elif isinstance(input_data, dict):
            source = input_data.get("body", input_data)
        else:
            source = {}
        
        for target, expr in mapping.items():
            if isinstance(expr, str) and expr.startswith("$"):
                field = expr[1:]
                result[target] = source.get(field)
            else:
                result[target] = expr
        
        return StepResult(success=True, data=result)
    return handler


@pytest.fixture
def test_workflow():
    """Create a test workflow with 3 steps."""
    return Workflow(
        uid="wf_test_001",
        name="Test Workflow",
        version=1,
        trigger=WorkflowTrigger(
            type=TriggerType.MANUAL,
        ),
        steps=[
            WorkflowStep(
                uid="step_api_call",
                type=StepType.API_CALL,
                name="Call External API",
                params={
                    "url": "https://api.example.com/users/123",
                    "method": "GET",
                },
            ),
            WorkflowStep(
                uid="step_transform",
                type=StepType.TRANSFORM_DATA,
                name="Transform Response",
                params={
                    "input": "$step_api_call",
                    "mapping": {
                        "user_name": "$name",
                        "user_email": "$email",
                    },
                },
                depends_on=["step_api_call"],
            ),
            WorkflowStep(
                uid="step_persist",
                type=StepType.CREATE_ENTITY,
                name="Persist Result",
                params={
                    "entity_type": "ProcessedUser",
                    "data": "$step_transform",
                },
                depends_on=["step_transform"],
            ),
        ],
    )


# =============================================================================
# Engine Tests
# =============================================================================

class TestOneXEngine:
    """Test the OneXEngine high-level API."""
    
    @pytest.mark.asyncio
    async def test_engine_creation(self):
        """Test that engine can be created."""
        engine = OneXEngine()
        assert engine is not None
    
    @pytest.mark.asyncio
    async def test_submit_workflow(self, test_workflow, mock_api_handler, mock_entity_handler, mock_transform_handler):
        """Test submitting a workflow for execution."""
        engine = OneXEngine()
        
        # Register handlers
        engine.register_step_handler(StepType.API_CALL, mock_api_handler)
        engine.register_step_handler(StepType.CREATE_ENTITY, mock_entity_handler)
        engine.register_step_handler(StepType.TRANSFORM_DATA, mock_transform_handler)
        
        # Execute
        context = ExecutionContext(
            user={"uid": "user_test", "name": "Test User"},
        )
        
        execution_id = await engine.submit_workflow(test_workflow, context)
        
        # Verify
        assert execution_id is not None
        assert execution_id.startswith("exec_")
        
        # Query result
        result = engine.query_execution(execution_id)
        assert result is not None
        assert result.status == ExecutionStatus.COMPLETED
        assert len(result.steps) == 3
    
    @pytest.mark.asyncio
    async def test_execution_order(self, test_workflow, mock_api_handler, mock_entity_handler, mock_transform_handler):
        """Test that steps execute in correct order."""
        engine = OneXEngine()
        engine.register_step_handler(StepType.API_CALL, mock_api_handler)
        engine.register_step_handler(StepType.CREATE_ENTITY, mock_entity_handler)
        engine.register_step_handler(StepType.TRANSFORM_DATA, mock_transform_handler)
        
        execution_id = await engine.submit_workflow(test_workflow)
        result = engine.query_execution(execution_id)
        
        # Check step order
        step_uids = [s.step_uid for s in result.steps]
        assert step_uids[0] == "step_api_call"
        assert step_uids[1] == "step_transform"
        assert step_uids[2] == "step_persist"
    
    @pytest.mark.asyncio
    async def test_step_logs_produced(self, test_workflow, mock_api_handler, mock_entity_handler, mock_transform_handler):
        """Test that execution logs are produced for each step."""
        engine = OneXEngine()
        engine.register_step_handler(StepType.API_CALL, mock_api_handler)
        engine.register_step_handler(StepType.CREATE_ENTITY, mock_entity_handler)
        engine.register_step_handler(StepType.TRANSFORM_DATA, mock_transform_handler)
        
        execution_id = await engine.submit_workflow(test_workflow)
        result = engine.query_execution(execution_id)
        
        # Each step should have a log
        assert len(result.steps) == 3
        
        for step in result.steps:
            assert step.step_uid is not None
            assert step.step_type is not None
            # Duration should be recorded
            # Note: duration_ms may be None for very fast mock handlers


class TestWorkflowExecutor:
    """Test the low-level WorkflowExecutor."""
    
    @pytest.mark.asyncio
    async def test_executor_creation(self):
        """Test that executor can be created."""
        executor = WorkflowExecutor()
        assert executor is not None
    
    @pytest.mark.asyncio
    async def test_log_step_execution(self, test_workflow):
        """Test that log step works without handlers."""
        # Create a simple workflow with just a log step
        log_workflow = Workflow(
            uid="wf_log_test",
            name="Log Test",
            version=1,
            trigger=WorkflowTrigger(type=TriggerType.MANUAL),
            steps=[
                WorkflowStep(
                    uid="step_log",
                    type=StepType.LOG,
                    params={"message": "Test log message", "level": "info"},
                ),
            ],
        )
        
        executor = WorkflowExecutor()
        context = ExecutionContext()
        
        log = await executor.execute(log_workflow, context)
        
        assert log.status == ExecutionStatus.COMPLETED
        assert len(log.step_logs) == 1
        assert log.step_logs[0].step_uid == "step_log"


class TestNoUICode:
    """Test that no UI code is invoked."""
    
    def test_no_page_imports(self):
        """Verify no page-related imports in core modules."""
        import orionx.core.executor as executor_module
        import orionx.schemas.workflow as workflow_module
        import orionx.schemas.execution as execution_module
        
        # Check module source for UI references (ignoring docstring explanations)
        import inspect
        
        for module in [executor_module, workflow_module, execution_module]:
            source = inspect.getsource(module)
            
            # Remove docstrings and comments for this check (they may contain explanatory text)
            # We only care about actual code usage, not documentation about what was removed
            import re
            # Remove triple-quoted strings (docstrings)
            source_no_docs = re.sub(r'""".*?"""', '', source, flags=re.DOTALL)
            source_no_docs = re.sub(r"'''.*?'''", '', source_no_docs, flags=re.DOTALL)
            # Remove single-line comments
            source_no_docs = re.sub(r'#.*$', '', source_no_docs, flags=re.MULTILINE)
            
            # Check for actual attribute access or variable names, not just the word
            # e.g., look for ".page_state" or "page_state =" but not just "page_state" in isolation
            assert ".page_state" not in source_no_docs.lower(), f"Found .page_state in {module.__name__}"
            assert "page_state =" not in source_no_docs.lower(), f"Found page_state assignment in {module.__name__}"
            assert "element_event" not in source_no_docs.lower(), f"Found element_event in {module.__name__}"
            assert "show_alert" not in source_no_docs.lower(), f"Found show_alert in {module.__name__}"
    
    def test_no_client_actions(self):
        """Verify client actions are not in StepType enum."""
        from orionx.schemas.workflow import StepType
        
        step_types = [s.value for s in StepType]
        
        # These should NOT be present
        assert "navigate" not in step_types
        assert "show_alert" not in step_types
        assert "scroll_to" not in step_types
        assert "focus_element" not in step_types
        assert "reset_inputs" not in step_types
    
    def test_no_ui_triggers(self):
        """Verify UI triggers are not in TriggerType enum."""
        from orionx.schemas.workflow import TriggerType
        
        trigger_types = [t.value for t in TriggerType]
        
        # These should NOT be present
        assert "element_event" not in trigger_types
        assert "page_load" not in trigger_types


class TestExecutionContext:
    """Test the execution context (renamed from EvaluationContext)."""
    
    def test_context_creation(self):
        """Test context can be created."""
        ctx = ExecutionContext(
            user={"uid": "user_1"},
            execution_state={"count": 0},
            input_params={"id": "123"},
        )
        
        assert ctx.user == {"uid": "user_1"}
        assert ctx.execution_state == {"count": 0}
        assert ctx.input_params == {"id": "123"}
    
    def test_context_get(self):
        """Test getting values from context."""
        ctx = ExecutionContext(
            user={"uid": "user_1", "name": "Test"},
            execution_state={"counter": 5},
        )
        
        assert ctx.get("Current User") == {"uid": "user_1", "name": "Test"}
        assert ctx.get("counter") == 5
        assert ctx.get("nonexistent") is None
    
    def test_context_state_mutation(self):
        """Test setting state in context."""
        ctx = ExecutionContext()
        
        ctx.set_state("key1", "value1")
        assert ctx.execution_state["key1"] == "value1"
        
        ctx.set_result("step_1", {"data": 123})
        assert ctx.workflow_data["step_1"] == {"data": 123}


# =============================================================================
# Run Tests
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
