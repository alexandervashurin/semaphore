.PHONY: build test clean install docs

# Go parameters
GOCMD=go
GOBUILD=$(GOCMD) build
GOTEST=$(GOCMD) test
GOGET=$(GOCMD) get
GOFMT=gofmt
GOLINT=golint

# Binary name
BINARY_NAME=terraform-provider-velum

# Build
build:
	$(GOBUILD) -o $(BINARY_NAME) -v

# Test
test:
	$(GOTEST) -v ./...

# Clean
clean:
	rm -f $(BINARY_NAME)
	rm -rf dist/

# Install locally
install: build
	mkdir -p ~/.terraform.d/plugins/registry.terraform.io/tnl-o/velum/0.1.0/$$(go env GOOS)_$$(go env GOARCH)
	cp $(BINARY_NAME) ~/.terraform.d/plugins/registry.terraform.io/tnl-o/velum/0.1.0/$$(go env GOOS)_$$(go env GOARCH)/

# Format code
fmt:
	$(GOFMT) -w .

# Lint
lint:
	$(GOLINT) ./...

# Download dependencies
deps:
	$(GOGET) ./...

# Build for all platforms
build-all: clean
	mkdir -p dist
	GOOS=linux GOARCH=amd64 $(GOBUILD) -o dist/$(BINARY_NAME)-linux-amd64 -v
	GOOS=linux GOARCH=arm64 $(GOBUILD) -o dist/$(BINARY_NAME)-linux-arm64 -v
	GOOS=darwin GOARCH=amd64 $(GOBUILD) -o dist/$(BINARY_NAME)-darwin-amd64 -v
	GOOS=darwin GOARCH=arm64 $(GOBUILD) -o dist/$(BINARY_NAME)-darwin-arm64 -v
	GOOS=windows GOARCH=amd64 $(GOBUILD) -o dist/$(BINARY_NAME)-windows-amd64.exe -v

# Generate docs
docs:
	@echo "Documentation is in README.md"

# Release (requires gh CLI)
release: build-all
	@echo "Creating release..."
	# gh release create v0.1.0 dist/* --title "v0.1.0" --notes "Initial release"

help:
	@echo "Available targets:"
	@echo "  build      - Build the provider"
	@echo "  test       - Run tests"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install locally for Terraform"
	@echo "  fmt        - Format code"
	@echo "  lint       - Run linter"
	@echo "  deps       - Download dependencies"
	@echo "  build-all  - Build for all platforms"
	@echo "  docs       - Generate documentation"
	@echo "  release    - Create GitHub release"
	@echo "  help       - Show this help"
