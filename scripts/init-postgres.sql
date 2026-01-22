-- Initialize PostgreSQL database for Elementa
-- This script is run automatically when the PostgreSQL container starts

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create development database
CREATE DATABASE elementa_dev;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE elementa TO elementa;
GRANT ALL PRIVILEGES ON DATABASE elementa_dev TO elementa;