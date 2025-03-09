default: help

.PHONY: help
help:
	@echo >&2 "docs - build docs"

.PHONY: docs
docs:
	mkdocs build
