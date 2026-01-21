"""
OneX Step Handlers

Server-side step handlers for workflow execution.
Refactored from OrionX to remove all client actions.

REMOVED (Client-side):
- set_state (UI state) - replaced with set_execution_state
- navigate (page navigation)
- show_alert (UI alerts)
- scroll_to (UI scrolling)
- focus_element (UI focus)
- reset_inputs (form reset)

KEPT (Server-side):
- create_entity (was create_thing)
- update_entity (was update_thing)
- delete_entity (was delete_thing)
- query_entity (new)
- send_email
- api_call
- schedule_workflow
- log
"""

from __future__ import annotations
from typing import Dict, Any, Optional, Callable, Awaitable
from dataclasses import dataclass
import logging
import aiohttp
import asyncio

from ..schemas.execution import ExecutionContext
from ..schemas.workflow import StepType


logger = logging.getLogger(__name__)


# =============================================================================
# Step Result
# =============================================================================

@dataclass
class StepResult:
    """Result of a step execution."""
    success: bool
    data: Optional[Any] = None
    error: Optional[str] = None
    
    def to_dict(self) -> Dict:
        return {
            "success": self.success,
            "data": self.data,
            "error": self.error,
        }


StepHandler = Callable[[Dict[str, Any], ExecutionContext], Awaitable[StepResult]]


# =============================================================================
# Step Handlers
# =============================================================================

class StepHandlers:
    """
    Handlers for server-side workflow steps.
    
    All step handlers are async and return StepResult.
    No client-side actions - everything runs on the server.
    """
    
    def __init__(
        self,
        db_session_factory=None,
        email_service=None,
        http_client: Optional[aiohttp.ClientSession] = None,
    ):
        self._db_factory = db_session_factory
        self._email = email_service
        self._http = http_client
        self._owns_http = http_client is None
    
    async def _get_http(self) -> aiohttp.ClientSession:
        """Get or create HTTP client."""
        if self._http is None:
            self._http = aiohttp.ClientSession()
        return self._http
    
    async def close(self) -> None:
        """Close resources."""
        if self._owns_http and self._http:
            await self._http.close()
            self._http = None
    
    # =========================================================================
    # Entity Operations
    # =========================================================================
    
    async def create_entity(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Create a database entity.
        
        Params:
            entity_type: Type name (e.g., "Order")
            data: Field values
        """
        entity_type = params.get("entity_type")
        data = params.get("data", {})
        
        if not entity_type:
            return StepResult(success=False, error="Missing entity_type")
        
        try:
            # Would use actual DB here
            logger.info(f"Created {entity_type}: {data}")
            return StepResult(
                success=True,
                data={"uid": f"{entity_type.lower()}_created", **data}
            )
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    async def update_entity(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Update a database entity.
        
        Params:
            entity_uid: UID of entity to update
            data: Field updates
        """
        entity_uid = params.get("entity_uid")
        data = params.get("data", {})
        
        if not entity_uid:
            return StepResult(success=False, error="Missing entity_uid")
        
        try:
            logger.info(f"Updated {entity_uid}: {data}")
            return StepResult(success=True, data={"uid": entity_uid, **data})
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    async def delete_entity(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Delete a database entity.
        
        Params:
            entity_uid: UID of entity to delete
        """
        entity_uid = params.get("entity_uid")
        
        if not entity_uid:
            return StepResult(success=False, error="Missing entity_uid")
        
        try:
            logger.info(f"Deleted {entity_uid}")
            return StepResult(success=True, data={"deleted": entity_uid})
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    async def query_entity(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Query database entities.
        
        Params:
            entity_type: Type to query
            filters: Query filters
            limit: Max results
            offset: Pagination offset
        """
        entity_type = params.get("entity_type")
        filters = params.get("filters", {})
        limit = params.get("limit", 100)
        offset = params.get("offset", 0)
        
        if not entity_type:
            return StepResult(success=False, error="Missing entity_type")
        
        try:
            # Would use actual DB here
            logger.info(f"Query {entity_type}: filters={filters}, limit={limit}")
            return StepResult(
                success=True,
                data={"results": [], "total": 0}
            )
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    # =========================================================================
    # External Communications
    # =========================================================================
    
    async def send_email(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Send an email.
        
        Params:
            to: Recipient email(s)
            subject: Email subject
            body: Email body (HTML or text)
            from_name: Sender name
        """
        to = params.get("to")
        subject = params.get("subject", "")
        body = params.get("body", "")
        
        if not to:
            return StepResult(success=False, error="Missing recipient")
        
        try:
            # Would use email service here
            logger.info(f"Email sent to {to}: {subject}")
            return StepResult(
                success=True,
                data={"sent_to": to, "subject": subject}
            )
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    async def api_call(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Make an external API call.
        
        Params:
            url: API endpoint
            method: HTTP method (GET, POST, etc.)
            headers: Request headers
            body: Request body (for POST/PUT)
            timeout: Request timeout in seconds
        """
        url = params.get("url")
        method = params.get("method", "GET").upper()
        headers = params.get("headers", {})
        body = params.get("body")
        timeout = params.get("timeout", 30)
        
        if not url:
            return StepResult(success=False, error="Missing URL")
        
        try:
            http = await self._get_http()
            async with http.request(
                method,
                url,
                headers=headers,
                json=body if method in ["POST", "PUT", "PATCH"] else None,
                timeout=aiohttp.ClientTimeout(total=timeout),
            ) as response:
                try:
                    data = await response.json()
                except:
                    data = await response.text()
                
                return StepResult(
                    success=response.status < 400,
                    data={
                        "status": response.status,
                        "body": data,
                    },
                    error=None if response.status < 400 else f"HTTP {response.status}",
                )
        except asyncio.TimeoutError:
            return StepResult(success=False, error="Request timed out")
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    # =========================================================================
    # Workflow Operations
    # =========================================================================
    
    async def schedule_workflow(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Schedule a workflow for later execution.
        
        Params:
            workflow_uid: Workflow to schedule
            delay_seconds: Delay before execution
            context: Context to pass to workflow
        """
        workflow_uid = params.get("workflow_uid")
        delay_seconds = params.get("delay_seconds", 0)
        
        if not workflow_uid:
            return StepResult(success=False, error="Missing workflow_uid")
        
        try:
            # Would use scheduler here
            logger.info(f"Scheduled {workflow_uid} in {delay_seconds}s")
            return StepResult(
                success=True,
                data={"scheduled": workflow_uid, "delay": delay_seconds}
            )
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    async def call_workflow(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Call another workflow synchronously.
        
        Params:
            workflow_uid: Workflow to call
            input_params: Parameters to pass
        """
        workflow_uid = params.get("workflow_uid")
        input_params = params.get("input_params", {})
        
        if not workflow_uid:
            return StepResult(success=False, error="Missing workflow_uid")
        
        try:
            # Would load and execute workflow here
            logger.info(f"Called {workflow_uid} with {input_params}")
            return StepResult(
                success=True,
                data={"called": workflow_uid}
            )
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    # =========================================================================
    # Data Transformation
    # =========================================================================
    
    async def transform_data(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Transform data using a mapping.
        
        Params:
            input: Input data
            mapping: Field mapping
        """
        input_data = params.get("input", {})
        mapping = params.get("mapping", {})
        
        try:
            result = {}
            for target_field, source_expr in mapping.items():
                if isinstance(source_expr, str) and source_expr.startswith("$"):
                    # Simple field reference
                    source_field = source_expr[1:]
                    result[target_field] = input_data.get(source_field)
                else:
                    result[target_field] = source_expr
            
            return StepResult(success=True, data=result)
        except Exception as e:
            return StepResult(success=False, error=str(e))
    
    async def validate_data(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Validate data against rules.
        
        Params:
            data: Data to validate
            rules: Validation rules
        """
        data = params.get("data", {})
        rules = params.get("rules", {})
        
        errors = []
        for field, rule in rules.items():
            value = data.get(field)
            
            if rule.get("required") and value is None:
                errors.append(f"{field} is required")
            
            if rule.get("min_length") and isinstance(value, str):
                if len(value) < rule["min_length"]:
                    errors.append(f"{field} must be at least {rule['min_length']} characters")
        
        if errors:
            return StepResult(success=False, error="; ".join(errors), data={"errors": errors})
        
        return StepResult(success=True, data=data)
    
    # =========================================================================
    # Logging
    # =========================================================================
    
    async def log(
        self,
        params: Dict[str, Any],
        context: ExecutionContext,
    ) -> StepResult:
        """
        Log a message.
        
        Params:
            message: Message to log
            level: Log level (debug, info, warning, error)
            data: Additional data to log
        """
        message = params.get("message", "")
        level = params.get("level", "info")
        data = params.get("data")
        
        log_fn = {
            "debug": logger.debug,
            "info": logger.info,
            "warning": logger.warning,
            "error": logger.error,
        }.get(level, logger.info)
        
        log_fn(f"[WorkflowLog] {message}" + (f" | data={data}" if data else ""))
        return StepResult(success=True, data={"logged": message})
    
    # =========================================================================
    # Handler Registry
    # =========================================================================
    
    def get_handlers(self) -> Dict[StepType, StepHandler]:
        """Get all step handlers."""
        return {
            StepType.CREATE_ENTITY: self.create_entity,
            StepType.UPDATE_ENTITY: self.update_entity,
            StepType.DELETE_ENTITY: self.delete_entity,
            StepType.QUERY_ENTITY: self.query_entity,
            StepType.SEND_EMAIL: self.send_email,
            StepType.API_CALL: self.api_call,
            StepType.SCHEDULE_WORKFLOW: self.schedule_workflow,
            StepType.CALL_WORKFLOW: self.call_workflow,
            StepType.TRANSFORM_DATA: self.transform_data,
            StepType.VALIDATE_DATA: self.validate_data,
            StepType.LOG: self.log,
        }
