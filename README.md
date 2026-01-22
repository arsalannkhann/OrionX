# Elementa Supply Chain Compliance System

Elementa is an AI-powered autonomous supply chain compliance agent that automates PFAS (Per- and polyfluoroalkyl substances) and chemical compliance data collection from suppliers. The system addresses the critical business challenge of achieving regulatory compliance with 2026 TSCA PFAS reporting requirements while overcoming "portal fatigue" through an email-first agentic workflow.

## Architecture

Elementa follows a microservices architecture built in Rust with the following core services:

- **API Gateway**: Central entry point handling authentication, routing, and request/response processing
- **Workflow Orchestration**: Manages complex multi-step compliance workflows
- **AI Agent**: Coordinates AI-powered decision making and task execution
- **Document Processing**: VLM-powered extraction and structuring of compliance documents
- **Email Communication**: Handles bidirectional email communication with suppliers
- **Chemical Database**: Manages PFAS identification and regulatory mapping
- **Compliance Data**: Stores and manages structured compliance records
- **Audit Trail**: Maintains immutable chain of custody documentation

## Technology Stack

- **Language**: Rust 2021 Edition
- **Web Framework**: Axum with Tokio async runtime
- **Databases**: PostgreSQL (compliance data), MongoDB (documents), Redis (caching)
- **Document Processing**: Vision-Language Models (VLM) for PDF/image extraction
- **Email**: SMTP/IMAP integration with template engine
- **Monitoring**: Prometheus metrics and structured logging
- **Deployment**: Docker containers with Kubernetes support

## Quick Start

### Prerequisites

- Rust 1.75 or later
- Docker and Docker Compose
- Make (optional, for convenience commands)

### Development Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd elementa
   ```

2. **Set up environment**
   ```bash
   make setup-dev
   ```
   This will:
   - Copy `.env.example` to `.env`
   - Start PostgreSQL, MongoDB, and Redis containers
   - Wait for databases to be ready

3. **Configure environment variables**
   Edit `.env` file with your actual API keys and configuration:
   ```bash
   # Required for VLM processing
   ELEMENTA__VLM__API_KEY=your-openai-api-key
   
   # Required for email functionality
   ELEMENTA__EMAIL__SMTP_USERNAME=your-email@gmail.com
   ELEMENTA__EMAIL__SMTP_PASSWORD=your-app-password
   ```

4. **Build and run**
   ```bash
   make build
   make run
   ```

5. **Verify installation**
   ```bash
   curl http://localhost:8080/health
   ```

### Using Docker Compose

For a complete environment with all services:

```bash
# Start all services
make docker-up

# View logs
make docker-logs

# Stop all services
make docker-down
```

## Development

### Project Structure

```
elementa/
├── services/           # Microservices
│   ├── api-gateway/   # Central API gateway
│   ├── workflow-orchestration/
│   ├── ai-agent/
│   ├── document-processing/
│   ├── email-communication/
│   ├── chemical-database/
│   ├── compliance-data/
│   └── audit-trail/
├── shared/            # Shared libraries
│   ├── models/        # Domain models
│   ├── database/      # Database utilities
│   └── utils/         # Common utilities
├── config/            # Configuration files
├── scripts/           # Setup and utility scripts
└── Cargo.toml         # Workspace configuration
```

### Available Commands

```bash
# Development
make build          # Build all services
make test           # Run all tests
make run            # Run API gateway
make check          # Run cargo check
make fmt            # Format code
make clippy         # Run linter

# Docker
make docker-up      # Start all services
make docker-down    # Stop all services
make docker-logs    # View logs

# Database
make migrate        # Run migrations
make reset-db       # Reset database (WARNING: deletes data)
```

### Testing

The project uses a dual testing approach:

- **Unit Tests**: Specific examples and edge cases
- **Property-Based Tests**: Universal correctness properties

```bash
# Run all tests
make test

# Run tests for specific service
cargo test -p elementa-api-gateway

# Run property-based tests with more iterations
PROPTEST_CASES=1000 cargo test
```

## Configuration

Configuration is managed through TOML files and environment variables:

- `config/default.toml`: Default configuration
- `config/development.toml`: Development overrides
- `config/production.toml`: Production settings (create as needed)
- `.env`: Environment-specific variables

Environment variables use the prefix `ELEMENTA__` with double underscores for nesting:
```bash
ELEMENTA__DATABASE__POSTGRES_URL=postgresql://...
ELEMENTA__EMAIL__SMTP_HOST=smtp.gmail.com
```

## API Documentation

Once running, the API gateway provides:

- Health check: `GET /health`
- Detailed health: `GET /api/v1/health/detailed`
- Metrics: `GET /metrics` (Prometheus format)

Full API documentation will be available at `/docs` once implemented.

## Monitoring

Elementa includes comprehensive monitoring:

- **Structured Logging**: JSON logs with request tracing
- **Metrics**: Prometheus-compatible metrics on port 9090
- **Health Checks**: Service health endpoints
- **Distributed Tracing**: Request correlation across services

## Security

- **Authentication**: Token-based authentication (JWT support planned)
- **Authorization**: Role-based access control
- **Data Encryption**: TLS in transit, encryption at rest for sensitive data
- **Audit Trail**: Immutable logging of all compliance-related actions
- **Input Validation**: Comprehensive validation of all inputs

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make test`
5. Run linting: `make clippy`
6. Format code: `make fmt`
7. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For questions or issues:
- Create an issue in the repository
- Check the documentation in `/docs`
- Review the configuration examples in `/config`

## Roadmap

Current implementation status:

- [x] Project setup and core infrastructure
- [ ] Core data models and database layer
- [ ] BOM processing service
- [ ] Chemical database integration
- [ ] Document processing (VLM integration)
- [ ] Email communication service
- [ ] Workflow orchestration engine
- [ ] Compliance dashboard and reporting
- [ ] Audit trail and security implementation
- [ ] Performance optimization and scalability
- [ ] Integration and system testing

See the tasks.md file in `.kiro/specs/elementa/` for detailed implementation plan.