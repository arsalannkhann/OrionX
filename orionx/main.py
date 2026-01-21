"""
OneX Main Application

FastAPI application entry point for the OneX execution engine.
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
import logging

from .api.routes import router


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)

# Create FastAPI app
app = FastAPI(
    title="OneX Execution Engine",
    description="""
## OneX - Pure Backend Workflow Execution Engine

OneX is a horizontal execution engine whose ONLY responsibility is:
- **Executing workflows**
- **Managing execution state**
- **Enforcing permissions**
- **Calling external APIs**

### What OneX Does NOT Do
- Generate UI
- Generate HTML / React
- Depend on frontend schemas
- Depend on any LLM or AI

### Public API Contract

**Inbound Operations:**
- `POST /api/v1/workflows/submit` - Submit a workflow for execution
- `GET /api/v1/executions/{id}` - Query execution status
- `POST /api/v1/executions/{id}/cancel` - Cancel a running workflow
- `POST /api/v1/executions/{id}/retry` - Retry a failed workflow

**Outbound Events (Webhooks):**
- `ExecutionStarted` - Workflow execution began
- `StepCompleted` - A step finished successfully
- `StepFailed` - A step failed
- `WorkflowCompleted` - Workflow finished successfully
- `WorkflowFailed` - Workflow failed

### Guarantees
1. **Deterministic execution** - Same inputs produce same execution order
2. **No UI knowledge** - Zero references to pages, elements, or frontend concepts
3. **No planning/reasoning** - Pure execution, no LLM or AI
4. **Idempotent retries** - Retrying produces consistent results
    """,
    version="0.1.0",
    docs_url="/docs",
    redoc_url="/redoc",
    openapi_url="/openapi.json",
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include router
app.include_router(router)


@app.on_event("startup")
async def startup_event():
    """Initialize on startup."""
    logging.info("OneX Execution Engine starting...")


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown."""
    logging.info("OneX Execution Engine shutting down...")


# Health check at root
@app.get("/", tags=["Root"])
async def root():
    """Root endpoint."""
    return {
        "name": "OneX Execution Engine",
        "version": "0.1.0",
        "docs": "/docs",
        "health": "/api/v1/health",
    }
