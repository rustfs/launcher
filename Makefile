.PHONY: help install-act test-ci test-ci-full clean pre-commit check-fmt check-clippy check-test check-frontend check-upstream

# Default target
help:
	@echo "RustFS Launcher - GitHub Actions Local Testing"
	@echo ""
	@echo "Available targets:"
	@echo "  make help          - Show this help message"
	@echo "  make pre-commit    - Run all pre-commit checks (fmt, clippy, frontend, tests)"
	@echo "  make check-upstream - Check for new upstream rustfs/rustfs versions"
	@echo ""
	@echo "Local checks:"
	@echo "  make check-fmt     - Check Rust code formatting"
	@echo "  make check-clippy  - Run Clippy linter"
	@echo "  make check-frontend - Build frontend"
	@echo "  make check-test    - Run all tests"
	@echo "  make fix-fmt       - Auto-fix Rust code formatting"
	@echo ""
	@echo "Act testing:"
	@echo "  make install-act   - Install act tool for local GitHub Actions testing"
	@echo "  make test-ci       - Run CI workflow locally (quick, Ubuntu only)"
	@echo "  make test-ci-full  - Run CI workflow with full checks"
	@echo "  make test-build    - Test build workflow file (validates syntax and logic)"
	@echo "  make test-build-verbose - Test build workflow with verbose output (for debugging)"
	@echo "  make list-jobs     - List all available jobs in workflows"
	@echo "  make clean         - Clean act cache and temporary files"
	@echo ""

# Install act tool
install-act:
	@echo "Installing act..."
	@if command -v brew >/dev/null 2>&1; then \
		brew install act; \
	else \
		echo "Error: Homebrew not found. Please install Homebrew first."; \
		echo "Visit: https://brew.sh"; \
		exit 1; \
	fi
	@echo "act installed successfully!"
	@act --version

# Check if act is installed
check-act:
	@command -v act >/dev/null 2>&1 || { \
		echo "Error: act is not installed. Run 'make install-act' first."; \
		exit 1; \
	}

# ============================================================================
# Pre-commit checks - Run all CI checks locally
# ============================================================================

# Run all pre-commit checks (matches CI workflow)
pre-commit: check-fmt check-clippy check-frontend check-test
	@echo ""
	@echo "=========================================="
	@echo "✅ All pre-commit checks passed!"
	@echo "=========================================="
	@echo "Your code is ready to commit and push."
	@echo ""

# Check Rust code formatting
check-fmt:
	@echo "=========================================="
	@echo "📝 Checking Rust code formatting..."
	@echo "=========================================="
	@cd src-tauri && cargo fmt --all --check
	@echo "✅ Formatting check passed!"
	@echo ""

# Run Clippy linter
check-clippy:
	@echo "=========================================="
	@echo "🔍 Running Clippy linter..."
	@echo "=========================================="
	@cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Clippy check passed!"
	@echo ""

# Build frontend
check-frontend:
	@echo "=========================================="
	@echo "🎨 Building frontend..."
	@echo "=========================================="
	@trunk build
	@echo "✅ Frontend build passed!"
	@echo ""

# Run all tests
check-test:
	@echo "=========================================="
	@echo "🧪 Running tests..."
	@echo "=========================================="
	@cd src-tauri && cargo test --all-features
	@echo "✅ All tests passed!"
	@echo ""

# Auto-fix Rust code formatting
fix-fmt:
	@echo "🔧 Auto-fixing Rust code formatting..."
	@cd src-tauri && cargo fmt --all
	@echo "✅ Code formatted!"

# ============================================================================
# Act testing - Run GitHub Actions locally
# ============================================================================


# Run CI workflow locally (quick test)
test-ci: check-act
	@echo "Running CI workflow locally..."
	@echo "Note: This uses Ubuntu container and may take a few minutes on first run."
	act push -W .github/workflows/ci.yml \
		--container-architecture linux/amd64 \
		--platform ubuntu-latest=catthehacker/ubuntu:act-latest

# Run CI workflow with full checks
test-ci-full: check-act
	@echo "Running CI workflow with all checks..."
	act push -W .github/workflows/ci.yml \
		--container-architecture linux/amd64 \
		--platform ubuntu-latest=catthehacker/ubuntu:full-latest

# Test build workflow (validates workflow file syntax and logic)
# Uses act to verify the build workflow file is correct
# Note: macOS/Windows build jobs will be skipped (require native runners),
#       but build-check job will run to validate workflow logic
test-build: check-act
	@echo "Testing build workflow file..."
	@echo "Note: This validates the workflow file syntax and logic"
	@echo "      macOS/Windows build jobs will be skipped (require native runners)"
	@echo ""
	act workflow_dispatch -W .github/workflows/build.yml \
		--container-architecture linux/amd64 \
		--container-options "--platform linux/amd64"

# Test build workflow with verbose output (for debugging)
test-build-verbose: check-act
	@echo "Testing build workflow with verbose output..."
	@echo "Note: macOS/Windows build jobs will be skipped"
	@echo ""
	act workflow_dispatch -W .github/workflows/build.yml \
		--container-architecture linux/amd64 \
		--container-options "--platform linux/amd64" \
		--verbose

# Test build in Docker (can compile and verify build process)
# This actually compiles the code in Docker, useful for CI validation
test-build-docker: check-act
	@echo "=========================================="
	@echo "🐳 Testing build compilation in Docker..."
	@echo "=========================================="
	@echo "This will run CI workflow to verify code compiles"
	@echo "Press Ctrl+C to cancel, or wait 3 seconds to continue..."
	@sleep 3
	act push -W .github/workflows/ci.yml \
		--container-architecture linux/amd64 \
		--platform ubuntu-latest=catthehacker/ubuntu:act-latest

# List all jobs in workflows
list-jobs: check-act
	@echo "=== CI Workflow Jobs ==="
	@act -W .github/workflows/ci.yml -l
	@echo ""
	@echo "=== Build Workflow Jobs ==="
	@act -W .github/workflows/build.yml -l

# Dry run - show what would be executed
dry-run-ci: check-act
	@echo "Dry run of CI workflow..."
	act push -W .github/workflows/ci.yml -n

dry-run-build: check-act
	@echo "Dry run of build workflow..."
	act workflow_dispatch -W .github/workflows/build.yml -n

# Clean act cache and temporary files
clean:
	@echo "Cleaning act cache..."
	@rm -rf ~/.cache/act
	@rm -rf /tmp/act-*
	@echo "Cache cleaned!"

# Check for new upstream versions
check-upstream:
	@echo "Checking upstream rustfs/rustfs for new versions..."
	@./scripts/check-upstream-version.sh

# Run specific job from CI workflow
test-ci-job: check-act
	@echo "Available jobs in CI workflow:"
	@act -W .github/workflows/ci.yml -l
	@echo ""
	@read -p "Enter job name to run: " job; \
	act push -W .github/workflows/ci.yml -j $$job

# Test with verbose output
test-ci-verbose: check-act
	@echo "Running CI workflow with verbose output..."
	act push -W .github/workflows/ci.yml \
		--container-architecture linux/amd64 \
		--platform ubuntu-latest=catthehacker/ubuntu:act-latest \
		--verbose

