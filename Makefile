# suppress some strange escape-codes
TERM := xterm-color
export TERM

cargo_src := $(shell find src -type f)
cargo_out := target/debug/rocd

images_src := $(wildcard docs/assets/dia/*.d2)
images_svg := $(patsubst %.d2,%.svg,$(images_src))

pwd := $(shell pwd)
uid := $(shell id -u)
gid := $(shell id -g)


default: all

.PHONY: help
help:
	@echo >&2 "Primary targets:"
	@echo >&2
	@echo >&2 "  all   - build rocd and run linters"
	@echo >&2 "  build - build rocd"
	@echo >&2 "  run   - (build and) run rocd"
	@echo >&2 "  docs  - build docs"
	@echo >&2 "  clean - clean artifacts"
	@echo >&2
	@echo >&2 "Auxiliary targets:"
	@echo >&2
	@echo >&2 "  docs-diagrams  - generate diagrams"
	@echo >&2 "  docs-docker    - build docs (for CI and locally without deps installation)"
	@echo >&2 "  docs-openapi   - bundle openapi.json into openapi.html"
	@echo >&2 "  docs-serve     - run mkdocs web-server with monitoring file changes"
	@echo >&2 "  docs-site      - build docs for site (using mkdocs)"
	@echo >&2 "  clean-diagrams - remove generated diagrams"
	@echo >&2 "  clean-all      - remove all temporary files (binaries, docs, LSP caches, ...)"
	@echo >&2 "  fmt            - format the source code (may use some unstable features!)"
	@echo >&2 "  lint           - run static Rust analyser and dry-run formatter"

#---------- build/run/test ----------#

.PHONY: all
all: build lint test

.PHONY: build
build: $(cargo_out) openapi/openapi.json

$(cargo_out): $(cargo_src)
	cargo build

openapi/openapi.json: $(cargo_out)
	cargo run -- --dump-openapi=json > openapi/openapi.json
	cargo run -- --dump-openapi=yaml > openapi/openapi.yaml

.PHONY: lint
lint: build
	cargo clippy
	cargo fmt -- --check

.PHONY: test
test: build
	cargo test

.PHONY: run
run:
	cargo run

.PHONY: fmt
fmt:
	cargo fmt -- \
	  --config unstable_features=true \
	  --config imports_granularity=Module \
	  --config normalize_doc_attributes=true \
	  --config comment_width=95

#---------- documentation ----------#

.PHONY: docs
docs: docs-diagrams docs-site docs-openapi

.PHONY: docs-docker
docs-docker:
	docker run --rm -t -v '$(pwd):$(pwd)' -w '$(pwd)' -u '$(uid):$(gid)' \
	  rocstreaming/env-sphinx \
	  make docs

.PHONY: docs-serve
docs-serve: docs-diagrams
	while :; do \
	  ( find docs ; echo mkdocs.yml ) | entr -drs \
	    '$(MAKE) --no-print-directory -i docs-diagrams && \
	    mkdocs serve --no-livereload'; \
	  pkill -s 0 mkdocs; \
	done

.PHONY: docs-diagrams
docs-diagrams: $(images_svg)

# Diagrams: generate SVG images from d2 sources.
#
# '--layout dagre' is used by default; skipping it allows to set layout on a
# per-diagram level.
# Other options: '--sketch', '--layout elk'.
%.svg : %.d2
	d2 --theme 0 --dark-theme 200 --pad 5 --scale 0.98 $<

.PHONY: docs-site
docs-site:
	mkdocs build

.PHONY: docs-openapi
docs-openapi:
	if openapi --version &>/dev/null ; then \
	  openapi build-docs -o openapi/openapi.html openapi/openapi.json ; \
	else \
	  echo "[WW] openapi tool not found; skipped bundling openapi.json" ; \
	fi

#---------- cleaning ----------#

.PHONY: clean
clean:
	rm -rf \
	  site/ \
	  target/
	rm -f \
	  openapi/openapi.json \
	  openapi/openapi.yaml \
	  openapi/openapi.html

.PHONY: clean-diagrams
clean-diagrams:
	rm -f \
	  docs/assets/dia/*.png \
	  docs/assets/dia/*.svg

.PHONY: clean-all
clean-all: clean
	rm -rf .cache/
