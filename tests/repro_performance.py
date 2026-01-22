
import pytest
import asyncio
import time
from orionx.core.executor import OneXEngine
from orionx.core.action_handlers import StepResult
from orionx.schemas.workflow import (
    Workflow,
    WorkflowStep,
    WorkflowTrigger,
    StepType,
    TriggerType,
)
from orionx.schemas.execution import ExecutionContext, ExecutionStatus

@pytest.fixture
def mock_sleep_handler():
    """Mock handler that sleeps for a specified duration."""
    async def handler(params, context):
        duration = params.get("duration", 0)
        await asyncio.sleep(duration)
        return StepResult(success=True, data={"slept": duration})
    return handler

@pytest.mark.asyncio
async def test_performance_generations(mock_sleep_handler):
    """
    Test to demonstrate performance issue with generational execution.

    Workflow:
    - Step A: 0.1s
    - Step B: 2.0s
    - Step C: 1.0s (Depends on A)

    Current behavior (Generational):
    1. Start A, B.
    2. Wait for A and B to finish (2.0s).
    3. Start C (1.0s).
    Total: ~3.0s

    Desired behavior (Streaming):
    1. Start A, B.
    2. A finishes (0.1s). C becomes ready.
    3. Start C.
    4. B finishes (2.0s). C finishes (1.1s total).
    Total: ~2.0s (determined by B)
    """
    engine = OneXEngine()
    engine.register_step_handler(StepType.API_CALL, mock_sleep_handler)

    workflow = Workflow(
        uid="wf_perf_test",
        name="Performance Test",
        version=1,
        trigger=WorkflowTrigger(type=TriggerType.MANUAL),
        steps=[
            WorkflowStep(
                uid="step_a",
                type=StepType.API_CALL,
                name="Step A",
                params={"duration": 0.1},
            ),
            WorkflowStep(
                uid="step_b",
                type=StepType.API_CALL,
                name="Step B",
                params={"duration": 2.0},
            ),
            WorkflowStep(
                uid="step_c",
                type=StepType.API_CALL,
                name="Step C",
                params={"duration": 1.0},
                depends_on=["step_a"],
            ),
        ],
    )

    start_time = time.time()
    execution_id = await engine.submit_workflow(workflow)
    end_time = time.time()

    duration = end_time - start_time
    print(f"\nWorkflow execution took {duration:.2f}s")

    # Check results
    result = engine.query_execution(execution_id)
    assert result.status == ExecutionStatus.COMPLETED

    # Assert performance
    # If optimization works, it should be close to 2.0s
    # If not, it will be close to 3.0s
    # We set a threshold of 2.5s. If it takes longer, the optimization is missing.
    if duration > 2.5:
        pytest.fail(f"Performance issue detected: Execution took {duration:.2f}s, expected < 2.5s")

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
