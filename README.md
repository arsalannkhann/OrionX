# OrionX Execution Engine

A pure backend workflow execution engine. No UI, no AI, just execution.

## What OrionX Does

- ✅ **Executes workflows** (DAG-based, parallel execution)
- ✅ **Manages execution state** (STEP, WORKFLOW, GLOBAL scopes)
- ✅ **Enforces permissions** (row-level, field-level)
- ✅ **Calls external APIs**
- ✅ **Provides REST API** (FastAPI with OpenAPI)

## What OrionX Does NOT Do

- ❌ Generate UI
- ❌ Generate HTML / React
- ❌ Depend on frontend schemas
- ❌ Depend on any LLM or AI

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/OrionX.git
cd OrionX

# Create virtual environment
python -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -e ".[dev]"
```

## Quick Start

### Run the Server

```bash
uvicorn orionx.main:app --reload
```

Then visit:
- Swagger UI: http://localhost:8000/docs
- ReDoc: http://localhost:8000/redoc

### Programmatic Usage

```python
from orionx import OrionXEngine, Workflow, WorkflowStep, StepType

# Create engine
engine = OrionXEngine()

# Define workflow
workflow = Workflow(
    uid="wf_example",
    name="Example Workflow",
    version=1,
    steps=[
        WorkflowStep(
            uid="step_log",
            type=StepType.LOG,
            params={"message": "Hello from OrionX!"}
        ),
    ],
)

# Execute
execution_id = await engine.submit_workflow(workflow)

# Query result
result = engine.query_execution(execution_id)
print(f"Status: {result.status}")
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/health` | Health check |
| `GET` | `/api/v1/schema` | Get step/trigger types |
| `POST` | `/api/v1/workflows/submit` | Submit workflow |
| `GET` | `/api/v1/executions/{id}` | Query execution |
| `POST` | `/api/v1/executions/{id}/cancel` | Cancel workflow |
| `POST` | `/api/v1/executions/{id}/retry` | Retry workflow |

## Running Tests

```bash
pytest tests/ -v
```

## License

MIT
