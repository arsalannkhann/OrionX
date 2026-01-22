"""
OrionX Persistence Tests

Tests for the execution state persistence layer.
Addresses audit finding: "Zero persistence for workflow execution state"
"""

import pytest
import asyncio
import os
from datetime import datetime
from pathlib import Path

from orionx.persistence.base import ExecutionRecord
from orionx.persistence.memory import InMemoryExecutionStore
from orionx.persistence.sqlite import SQLiteExecutionStore
from orionx.schemas.execution import ExecutionStatus
from orionx.config import get_config, reset_config, load_config


class TestInMemoryStore:
    """Test in-memory execution store."""
    
    @pytest.fixture
    def store(self):
        return InMemoryExecutionStore()
    
    @pytest.fixture
    def sample_record(self):
        return ExecutionRecord(
            execution_id="exec_test_001",
            workflow_uid="wf_test",
            user_uid="user_123",
            status=ExecutionStatus.RUNNING,
            started_at=datetime.utcnow(),
        )
    
    @pytest.mark.asyncio
    async def test_save_and_get(self, store, sample_record):
        """Test saving and retrieving a record."""
        await store.save(sample_record)
        
        retrieved = await store.get(sample_record.execution_id)
        
        assert retrieved is not None
        assert retrieved.execution_id == sample_record.execution_id
        assert retrieved.workflow_uid == sample_record.workflow_uid
        assert retrieved.status == ExecutionStatus.RUNNING
    
    @pytest.mark.asyncio
    async def test_update_status(self, store, sample_record):
        """Test updating execution status."""
        await store.save(sample_record)
        
        completed_at = datetime.utcnow()
        await store.update_status(
            sample_record.execution_id,
            ExecutionStatus.COMPLETED,
            completed_at=completed_at,
        )
        
        retrieved = await store.get(sample_record.execution_id)
        assert retrieved.status == ExecutionStatus.COMPLETED
        assert retrieved.completed_at is not None
    
    @pytest.mark.asyncio
    async def test_append_step_log(self, store, sample_record):
        """Test appending step logs."""
        await store.save(sample_record)
        
        step_log = {"step_uid": "step_1", "status": "completed"}
        await store.append_step_log(sample_record.execution_id, step_log)
        
        retrieved = await store.get(sample_record.execution_id)
        assert len(retrieved.step_logs) == 1
        assert retrieved.step_logs[0]["step_uid"] == "step_1"
    
    @pytest.mark.asyncio
    async def test_list_active(self, store):
        """Test listing active executions."""
        # Create multiple records
        for i, status in enumerate([
            ExecutionStatus.PENDING,
            ExecutionStatus.RUNNING,
            ExecutionStatus.COMPLETED,
            ExecutionStatus.FAILED,
        ]):
            record = ExecutionRecord(
                execution_id=f"exec_{i}",
                workflow_uid="wf_test",
                user_uid=None,
                status=status,
                started_at=datetime.utcnow(),
            )
            await store.save(record)
        
        active = await store.list_active()
        
        # Should only return PENDING and RUNNING
        assert len(active) == 2
        statuses = {r.status for r in active}
        assert statuses == {ExecutionStatus.PENDING, ExecutionStatus.RUNNING}
    
    @pytest.mark.asyncio
    async def test_delete(self, store, sample_record):
        """Test deleting a record."""
        await store.save(sample_record)
        
        deleted = await store.delete(sample_record.execution_id)
        assert deleted is True
        
        retrieved = await store.get(sample_record.execution_id)
        assert retrieved is None


class TestSQLiteStore:
    """Test SQLite execution store."""
    
    @pytest.fixture
    def db_path(self, tmp_path):
        return str(tmp_path / "test_executions.db")
    
    @pytest.fixture
    def store(self, db_path):
        return SQLiteExecutionStore(db_path)
    
    @pytest.fixture
    def sample_record(self):
        return ExecutionRecord(
            execution_id="exec_sqlite_001",
            workflow_uid="wf_test",
            user_uid="user_456",
            status=ExecutionStatus.PENDING,
            started_at=datetime.utcnow(),
        )
    
    @pytest.mark.asyncio
    async def test_persistence_across_instances(self, db_path, sample_record):
        """Test that data persists across store instances (simulates restart)."""
        # Save with first instance
        store1 = SQLiteExecutionStore(db_path)
        await store1.save(sample_record)
        
        # Retrieve with second instance (simulates server restart)
        store2 = SQLiteExecutionStore(db_path)
        retrieved = await store2.get(sample_record.execution_id)
        
        assert retrieved is not None
        assert retrieved.execution_id == sample_record.execution_id
    
    @pytest.mark.asyncio
    async def test_save_and_get(self, store, sample_record):
        """Test saving and retrieving a record."""
        await store.save(sample_record)
        
        retrieved = await store.get(sample_record.execution_id)
        
        assert retrieved is not None
        assert retrieved.workflow_uid == "wf_test"
        assert retrieved.user_uid == "user_456"
    
    @pytest.mark.asyncio
    async def test_complex_data_serialization(self, store):
        """Test that complex data (step logs, errors) serialize correctly."""
        record = ExecutionRecord(
            execution_id="exec_complex",
            workflow_uid="wf_test",
            user_uid=None,
            status=ExecutionStatus.FAILED,
            started_at=datetime.utcnow(),
            completed_at=datetime.utcnow(),
            step_logs=[
                {"step_uid": "step_1", "result": {"data": [1, 2, 3]}},
                {"step_uid": "step_2", "error": "Something failed"},
            ],
            error={"type": "RuntimeError", "message": "Test error"},
            input_snapshot={"user": {"name": "Test"}, "params": {"id": 123}},
        )
        
        await store.save(record)
        retrieved = await store.get(record.execution_id)
        
        assert len(retrieved.step_logs) == 2
        assert retrieved.step_logs[0]["result"]["data"] == [1, 2, 3]
        assert retrieved.error["type"] == "RuntimeError"
        assert retrieved.input_snapshot["params"]["id"] == 123


class TestConfig:
    """Test configuration loading."""
    
    def setup_method(self):
        """Reset config before each test."""
        reset_config()
    
    def teardown_method(self):
        """Reset config after each test."""
        reset_config()
    
    def test_default_config(self):
        """Test default configuration values."""
        config = load_config()
        
        assert config.limits.max_db_queries == 100
        assert config.limits.max_api_calls == 10
        assert config.limits.max_steps == 100
        assert config.limits.workflow_timeout_seconds == 300.0
    
    def test_config_from_env(self, monkeypatch):
        """Test configuration from environment variables."""
        monkeypatch.setenv("ORIONX_MAX_DB_QUERIES", "500")
        monkeypatch.setenv("ORIONX_MAX_API_CALLS", "50")
        monkeypatch.setenv("ORIONX_WORKFLOW_TIMEOUT", "600")
        
        reset_config()
        config = load_config()
        
        assert config.limits.max_db_queries == 500
        assert config.limits.max_api_calls == 50
        assert config.limits.workflow_timeout_seconds == 600.0
    
    def test_persistence_backend_config(self, monkeypatch):
        """Test persistence backend configuration."""
        from orionx.config import PersistenceBackend
        
        monkeypatch.setenv("ORIONX_PERSISTENCE_BACKEND", "redis")
        monkeypatch.setenv("ORIONX_REDIS_URL", "redis://localhost:6379/1")
        
        reset_config()
        config = load_config()
        
        assert config.persistence.backend == PersistenceBackend.REDIS
        assert config.persistence.redis_url == "redis://localhost:6379/1"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
