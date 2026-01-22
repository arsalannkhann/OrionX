"""
SQLite Execution Store

File-based persistent storage for production use.
Addresses audit finding: "Zero persistence for workflow execution state"
"""

import json
import sqlite3
import asyncio
from typing import List, Optional
from datetime import datetime
from pathlib import Path

from .base import ExecutionStore, ExecutionRecord
from ..schemas.execution import ExecutionStatus


class SQLiteExecutionStore(ExecutionStore):
    """
    SQLite-backed execution store.
    
    Features:
    - File-based persistence (survives server restarts)
    - Automatic table creation
    - JSON serialization for complex fields
    - Thread-safe via asyncio executor
    
    Use for:
    - Single-server production deployments
    - Development with persistence
    - Testing persistence behavior
    """
    
    CREATE_TABLE_SQL = """
    CREATE TABLE IF NOT EXISTS executions (
        execution_id TEXT PRIMARY KEY,
        workflow_uid TEXT NOT NULL,
        user_uid TEXT,
        status TEXT NOT NULL,
        started_at TEXT NOT NULL,
        completed_at TEXT,
        step_logs TEXT NOT NULL DEFAULT '[]',
        error TEXT,
        input_snapshot TEXT,
        output TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    );
    
    CREATE INDEX IF NOT EXISTS idx_executions_status ON executions(status);
    CREATE INDEX IF NOT EXISTS idx_executions_workflow ON executions(workflow_uid);
    CREATE INDEX IF NOT EXISTS idx_executions_user ON executions(user_uid);
    """
    
    def __init__(self, db_path: str = "./orionx_executions.db"):
        self.db_path = db_path
        self._ensure_db()
    
    def _ensure_db(self) -> None:
        """Ensure database and tables exist."""
        Path(self.db_path).parent.mkdir(parents=True, exist_ok=True)
        with sqlite3.connect(self.db_path) as conn:
            conn.executescript(self.CREATE_TABLE_SQL)
    
    def _get_connection(self) -> sqlite3.Connection:
        """Get a new database connection."""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        return conn
    
    async def save(self, record: ExecutionRecord) -> None:
        """Save or update an execution record."""
        def _save():
            with self._get_connection() as conn:
                conn.execute("""
                    INSERT OR REPLACE INTO executions 
                    (execution_id, workflow_uid, user_uid, status, started_at, 
                     completed_at, step_logs, error, input_snapshot, output, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, (
                    record.execution_id,
                    record.workflow_uid,
                    record.user_uid,
                    record.status.value,
                    record.started_at.isoformat(),
                    record.completed_at.isoformat() if record.completed_at else None,
                    json.dumps(record.step_logs),
                    json.dumps(record.error) if record.error else None,
                    json.dumps(record.input_snapshot) if record.input_snapshot else None,
                    json.dumps(record.output) if record.output else None,
                    datetime.utcnow().isoformat(),
                ))
        
        await asyncio.get_event_loop().run_in_executor(None, _save)
    
    async def get(self, execution_id: str) -> Optional[ExecutionRecord]:
        """Get an execution record by ID."""
        def _get():
            with self._get_connection() as conn:
                row = conn.execute(
                    "SELECT * FROM executions WHERE execution_id = ?",
                    (execution_id,)
                ).fetchone()
                return self._row_to_record(row) if row else None
        
        return await asyncio.get_event_loop().run_in_executor(None, _get)
    
    async def delete(self, execution_id: str) -> bool:
        """Delete an execution record."""
        def _delete():
            with self._get_connection() as conn:
                cursor = conn.execute(
                    "DELETE FROM executions WHERE execution_id = ?",
                    (execution_id,)
                )
                return cursor.rowcount > 0
        
        return await asyncio.get_event_loop().run_in_executor(None, _delete)
    
    async def list_by_status(self, status: ExecutionStatus) -> List[ExecutionRecord]:
        """List executions with a given status."""
        def _list():
            with self._get_connection() as conn:
                rows = conn.execute(
                    "SELECT * FROM executions WHERE status = ? ORDER BY started_at DESC",
                    (status.value,)
                ).fetchall()
                return [self._row_to_record(row) for row in rows]
        
        return await asyncio.get_event_loop().run_in_executor(None, _list)
    
    async def list_active(self) -> List[ExecutionRecord]:
        """List all active (running/pending) executions."""
        def _list():
            with self._get_connection() as conn:
                rows = conn.execute(
                    "SELECT * FROM executions WHERE status IN (?, ?) ORDER BY started_at DESC",
                    (ExecutionStatus.PENDING.value, ExecutionStatus.RUNNING.value)
                ).fetchall()
                return [self._row_to_record(row) for row in rows]
        
        return await asyncio.get_event_loop().run_in_executor(None, _list)
    
    async def update_status(
        self,
        execution_id: str,
        status: ExecutionStatus,
        error: Optional[dict] = None,
        completed_at: Optional[datetime] = None,
    ) -> bool:
        """Update execution status."""
        def _update():
            with self._get_connection() as conn:
                if error is not None and completed_at is not None:
                    cursor = conn.execute(
                        "UPDATE executions SET status = ?, error = ?, completed_at = ?, updated_at = ? WHERE execution_id = ?",
                        (status.value, json.dumps(error), completed_at.isoformat(), datetime.utcnow().isoformat(), execution_id)
                    )
                elif completed_at is not None:
                    cursor = conn.execute(
                        "UPDATE executions SET status = ?, completed_at = ?, updated_at = ? WHERE execution_id = ?",
                        (status.value, completed_at.isoformat(), datetime.utcnow().isoformat(), execution_id)
                    )
                elif error is not None:
                    cursor = conn.execute(
                        "UPDATE executions SET status = ?, error = ?, updated_at = ? WHERE execution_id = ?",
                        (status.value, json.dumps(error), datetime.utcnow().isoformat(), execution_id)
                    )
                else:
                    cursor = conn.execute(
                        "UPDATE executions SET status = ?, updated_at = ? WHERE execution_id = ?",
                        (status.value, datetime.utcnow().isoformat(), execution_id)
                    )
                return cursor.rowcount > 0
        
        return await asyncio.get_event_loop().run_in_executor(None, _update)
    
    async def append_step_log(
        self,
        execution_id: str,
        step_log: dict,
    ) -> bool:
        """Append a step log to an execution."""
        def _append():
            with self._get_connection() as conn:
                # Get current step_logs
                row = conn.execute(
                    "SELECT step_logs FROM executions WHERE execution_id = ?",
                    (execution_id,)
                ).fetchone()
                
                if not row:
                    return False
                
                current_logs = json.loads(row["step_logs"])
                current_logs.append(step_log)
                
                cursor = conn.execute(
                    "UPDATE executions SET step_logs = ?, updated_at = ? WHERE execution_id = ?",
                    (json.dumps(current_logs), datetime.utcnow().isoformat(), execution_id)
                )
                return cursor.rowcount > 0
        
        return await asyncio.get_event_loop().run_in_executor(None, _append)
    
    async def list_for_workflow(self, workflow_uid: str, limit: int = 100) -> List[ExecutionRecord]:
        """List recent executions for a workflow."""
        def _list():
            with self._get_connection() as conn:
                rows = conn.execute(
                    "SELECT * FROM executions WHERE workflow_uid = ? ORDER BY started_at DESC LIMIT ?",
                    (workflow_uid, limit)
                ).fetchall()
                return [self._row_to_record(row) for row in rows]
        
        return await asyncio.get_event_loop().run_in_executor(None, _list)
    
    async def list_for_user(self, user_uid: str, limit: int = 100) -> List[ExecutionRecord]:
        """List recent executions for a user."""
        def _list():
            with self._get_connection() as conn:
                rows = conn.execute(
                    "SELECT * FROM executions WHERE user_uid = ? ORDER BY started_at DESC LIMIT ?",
                    (user_uid, limit)
                ).fetchall()
                return [self._row_to_record(row) for row in rows]
        
        return await asyncio.get_event_loop().run_in_executor(None, _list)
    
    def _row_to_record(self, row: sqlite3.Row) -> ExecutionRecord:
        """Convert a database row to an ExecutionRecord."""
        return ExecutionRecord(
            execution_id=row["execution_id"],
            workflow_uid=row["workflow_uid"],
            user_uid=row["user_uid"],
            status=ExecutionStatus(row["status"]),
            started_at=datetime.fromisoformat(row["started_at"]),
            completed_at=datetime.fromisoformat(row["completed_at"]) if row["completed_at"] else None,
            step_logs=json.loads(row["step_logs"]),
            error=json.loads(row["error"]) if row["error"] else None,
            input_snapshot=json.loads(row["input_snapshot"]) if row["input_snapshot"] else None,
            output=json.loads(row["output"]) if row["output"] else None,
        )
