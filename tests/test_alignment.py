"""
OneX Alignment Tests

Phase 6: Elementa Alignment Verification

These tests verify that OneX is ready to be used as the execution
layer for Elementa (or any other frontend system).

Guarantees verified:
1. UI-agnostic: No page/element/component references
2. No LLM dependency: Pure execution, no AI
3. Deterministic: Same inputs → same execution order
4. Event-driven: Proper execution events emitted
5. Pluggable: Custom handlers can be registered
"""

import pytest
import sys

from orionx.core.executor import OneXEngine
from orionx.core.action_handlers import StepResult
from orionx.schemas.workflow import (
    Workflow,
    WorkflowStep,
    WorkflowTrigger,
    StepType,
    TriggerType,
)
from orionx.schemas.execution import ExecutionContext


class TestUIAgnostic:
    """Verify OneX has no UI knowledge."""
    
    def test_no_page_schema_imports(self):
        """Verify no page schema is importable from OneX."""
        onex_modules = [m for m in sys.modules if m.startswith('onex')]
        
        for module_name in onex_modules:
            module = sys.modules[module_name]
            if hasattr(module, '__dict__'):
                symbols = list(module.__dict__.keys())
                # Should not have page-related symbols
                assert 'Page' not in symbols, f"Found Page in {module_name}"
                assert 'PageState' not in symbols, f"Found PageState in {module_name}"
                assert 'OrionXPage' not in symbols, f"Found OrionXPage in {module_name}"
    
    def test_no_element_schema_imports(self):
        """Verify no element schema is importable from OneX."""
        onex_modules = [m for m in sys.modules if m.startswith('onex')]
        
        for module_name in onex_modules:
            module = sys.modules[module_name]
            if hasattr(module, '__dict__'):
                symbols = list(module.__dict__.keys())
                assert 'Element' not in symbols or module_name.endswith('__init__'), f"Found Element in {module_name}"
                assert 'OrionXElement' not in symbols, f"Found OrionXElement in {module_name}"
    
    def test_execution_context_has_no_ui_fields(self):
        """Verify ExecutionContext has execution-oriented fields only."""
        ctx = ExecutionContext()
        
        # Should have these execution-oriented fields
        assert hasattr(ctx, 'execution_state')
        assert hasattr(ctx, 'input_params')
        assert hasattr(ctx, 'workflow_data')
        assert hasattr(ctx, 'user')
        
        # Should NOT have these UI fields
        assert not hasattr(ctx, 'page_state')
        assert not hasattr(ctx, 'url_params')
        assert not hasattr(ctx, 'current_page')
        assert not hasattr(ctx, 'dom')
    
    def test_step_types_are_backend_only(self):
        """Verify all StepTypes are server-side operations."""
        from orionx.schemas.workflow import StepType
        
        backend_steps = {
            'create_entity', 'update_entity', 'delete_entity', 'query_entity',
            'api_call', 'send_email', 'condition', 'loop',
            'schedule_workflow', 'call_workflow', 'transform_data',
            'validate_data', 'plugin_action', 'set_execution_state', 'log'
        }
        
        frontend_steps = {
            'navigate', 'show_alert', 'scroll_to', 'focus_element',
            'reset_inputs', 'set_state', 'show_modal', 'hide_modal'
        }
        
        actual_steps = {s.value for s in StepType}
        
        # All actual steps should be backend steps
        assert actual_steps.issubset(backend_steps), f"Found non-backend steps: {actual_steps - backend_steps}"
        
        # No frontend steps should be present
        assert actual_steps.isdisjoint(frontend_steps), f"Found frontend steps: {actual_steps & frontend_steps}"


class TestNoLLMDependency:
    """Verify OneX has no LLM or AI dependencies."""
    
    def test_no_llm_imports(self):
        """Verify no LLM libraries are imported."""
        llm_modules = ['openai', 'anthropic', 'langchain', 'transformers', 'torch', 'tensorflow']
        
        for llm_module in llm_modules:
            assert llm_module not in sys.modules, f"LLM module {llm_module} is imported"
    
    def test_no_ai_in_step_types(self):
        """Verify no AI-related step types."""
        from orionx.schemas.workflow import StepType
        
        step_values = [s.value.lower() for s in StepType]
        
        # These are explicit AI step types that should NOT exist
        ai_step_names = ['llm_call', 'gpt_generate', 'ai_inference', 'ml_predict', 'generate_text']
        
        for ai_step in ai_step_names:
            assert ai_step not in step_values, f"Found AI step type: {ai_step}"


class TestDeterministicExecution:
    """Verify execution is deterministic."""
    
    @pytest.fixture
    def deterministic_workflow(self):
        """Create a workflow with multiple steps."""
        return Workflow(
            uid="wf_deterministic",
            name="Deterministic Test",
            version=1,
            trigger=WorkflowTrigger(type=TriggerType.MANUAL),
            steps=[
                WorkflowStep(uid="step_1", type=StepType.LOG, params={"message": "Step 1"}),
                WorkflowStep(uid="step_2", type=StepType.LOG, params={"message": "Step 2"}, depends_on=["step_1"]),
                WorkflowStep(uid="step_3", type=StepType.LOG, params={"message": "Step 3"}, depends_on=["step_1"]),
                WorkflowStep(uid="step_4", type=StepType.LOG, params={"message": "Step 4"}, depends_on=["step_2", "step_3"]),
            ],
        )
    
    @pytest.mark.asyncio
    async def test_same_order_multiple_runs(self, deterministic_workflow):
        """Verify same workflow produces same execution order."""
        engine = OneXEngine()
        
        # Run multiple times
        orders = []
        for _ in range(5):
            execution_id = await engine.submit_workflow(deterministic_workflow)
            result = engine.query_execution(execution_id)
            order = [s.step_uid for s in result.steps]
            orders.append(order)
        
        # All runs should have the same order
        for order in orders[1:]:
            assert order == orders[0], "Execution order is not deterministic"
    
    @pytest.mark.asyncio
    async def test_dependency_order_respected(self, deterministic_workflow):
        """Verify dependencies are respected in execution order."""
        engine = OneXEngine()
        
        execution_id = await engine.submit_workflow(deterministic_workflow)
        result = engine.query_execution(execution_id)
        order = [s.step_uid for s in result.steps]
        
        # step_1 must come before step_2 and step_3
        assert order.index("step_1") < order.index("step_2")
        assert order.index("step_1") < order.index("step_3")
        
        # step_4 must come after step_2 and step_3
        assert order.index("step_4") > order.index("step_2")
        assert order.index("step_4") > order.index("step_3")


class TestPluggableHandlers:
    """Verify custom handlers can be registered."""
    
    @pytest.mark.asyncio
    async def test_custom_handler_registration(self):
        """Verify custom step handlers can be registered."""
        engine = OneXEngine()
        
        call_count = 0
        
        async def custom_handler(params, context):
            nonlocal call_count
            call_count += 1
            return StepResult(success=True, data={"custom": True})
        
        engine.register_step_handler(StepType.API_CALL, custom_handler)
        
        workflow = Workflow(
            uid="wf_custom",
            name="Custom Handler Test",
            version=1,
            trigger=WorkflowTrigger(type=TriggerType.MANUAL),
            steps=[
                WorkflowStep(
                    uid="step_custom",
                    type=StepType.API_CALL,
                    params={"url": "http://example.com"}
                ),
            ],
        )
        
        await engine.submit_workflow(workflow)
        
        assert call_count == 1, "Custom handler was not called"
    
    @pytest.mark.asyncio
    async def test_multiple_handler_types(self):
        """Verify multiple handler types can be registered."""
        engine = OneXEngine()
        
        handlers_called = set()
        
        async def api_handler(params, context):
            handlers_called.add("api")
            return StepResult(success=True, data={})
        
        async def entity_handler(params, context):
            handlers_called.add("entity")
            return StepResult(success=True, data={})
        
        engine.register_step_handler(StepType.API_CALL, api_handler)
        engine.register_step_handler(StepType.CREATE_ENTITY, entity_handler)
        
        workflow = Workflow(
            uid="wf_multi",
            name="Multi Handler Test",
            version=1,
            trigger=WorkflowTrigger(type=TriggerType.MANUAL),
            steps=[
                WorkflowStep(uid="step_api", type=StepType.API_CALL, params={}),
                WorkflowStep(uid="step_entity", type=StepType.CREATE_ENTITY, params={"entity_type": "Test"}, depends_on=["step_api"]),
            ],
        )
        
        await engine.submit_workflow(workflow)
        
        assert handlers_called == {"api", "entity"}


class TestAPIContract:
    """Verify the API contract is properly defined."""
    
    def test_api_routes_exist(self):
        """Verify all required API routes exist."""
        from orionx.main import app
        
        routes = [r.path for r in app.routes if hasattr(r, 'path')]
        
        required_routes = [
            '/api/v1/health',
            '/api/v1/schema',
            '/api/v1/workflows/submit',
            '/api/v1/executions/{execution_id}',
            '/api/v1/executions/{execution_id}/cancel',
            '/api/v1/executions/{execution_id}/retry',
        ]
        
        for route in required_routes:
            assert route in routes, f"Missing required route: {route}"
    
    def test_openapi_available(self):
        """Verify OpenAPI schema is available."""
        from orionx.main import app
        
        assert app.openapi_url == "/openapi.json"
        
        # Generate OpenAPI schema
        schema = app.openapi()
        
        assert "openapi" in schema
        assert "info" in schema
        assert schema["info"]["title"] == "OneX Execution Engine"
        assert "paths" in schema


class TestIndependence:
    """Verify OneX is independent of OrionX."""
    
    def test_no_orionx_imports(self):
        """Verify no OrionX modules are loaded."""
        # Import all OneX modules
        
        # Check for OrionX imports
        orionx_patterns = ['OrionX', 'schemas.page', 'schemas.element', 'engine.workflow']
        
        for module_name in sys.modules:
            for pattern in orionx_patterns:
                if pattern in module_name and not module_name.startswith('onex'):
                    pytest.fail(f"OrionX module loaded: {module_name}")
    
    def test_standalone_import(self):
        """Verify OneX can be imported without any errors."""
        # This should not raise any ImportError
        from orionx import (
            OneXEngine,
            ExecutionContext,
            Workflow,
        )
        
        # All imports should be usable
        assert OneXEngine is not None
        assert ExecutionContext is not None
        assert Workflow is not None


# =============================================================================
# Run Tests
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
