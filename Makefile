cargo_src := $(shell find src -type f)
cargo_out := target/debug/rocd

images_src := $(wildcard docs/assets/dia/*.d2)
images_svg := $(patsubst %.d2,%.svg,$(images_src))

temp_images := \
  docs/assets/dia/*.png \
  docs/assets/dia/*.svg
temp_files := \
  site/ \
  target/ \
  rest/openapi.html
temp_dirs_aux := \
  .cache/

pwd := $(shell pwd)
uid := $(shell id -u)
gid := $(shell id -g)

# XXX
unstable_rustfmt_opts := \
  --config unstable_features=true \
  --config imports_granularity=Module \
  --config normalize_doc_attributes=true \
  --config comment_width=100
#  --config struct_field_align_threshold=30


default: help

.PHONY: help
help:
	@echo >&2 "Primary targets:"
	@echo >&2
	@echo >&2 "  build - build rocd"
	@echo >&2 "  run   - (build and) run rocd"
	@echo >&2 "  docs  - build docs"
	@echo >&2 "  clean - clean artifacts"
	@echo >&2
	@echo >&2 "Auxiliary targets:"
	@echo >&2
	@echo >&2 "  docs-diagrams  - generate diagrams"
	@echo >&2 "  docs-serve     - run mkdocs web-server with monitoring file changes"
	@echo >&2 "  clean-diagrams - remove generated diagrams"
	@echo >&2 "  clean-all      - remove all temporary files (binaries, docs, LSP caches, ...)"
	@echo >&2 "  fmt            - format the source code (may use some unstable features!)"

#---------- build/run/test ----------#

.PHONY: build
build: $(cargo_out) rest/openapi.json

$(cargo_out): $(cargo_src)
	cargo build

.PHONY: run
run:
	cargo run

.PHONY: fmt
fmt:
	cargo fmt -- $(unstable_rustfmt_opts)

#---------- openapi ----------#

rest/openapi.json: $(cargo_out)
	cargo run rocd -- --dump-openapi > rest/openapi.json

rest/openapi.html: rest/openapi.json
	openapi build-docs -o $@ $<

#---------- documentation ----------#

# chmod is a fix for mkdocs_puml plugin on Nixos: site/assets has u-w permission
# possible reason:
# - https://github.com/MikhailKravets/mkdocs_puml/blob/2.3.0/mkdocs_puml/plugin.py#L222
# - see `stat /nix/store/*-mkdocs_puml-*/lib/python*/site-packages/mkdocs_puml/static/`
# TODO: report to upstream
.PHONY: docs
docs: docs-diagrams docs-openapi
	mkdocs build
	#if [ -d site ]; then chmod -R u+w site ; fi

.PHONY: docs-docker
docs-docker:
	docker run --rm -t -v '$(pwd):$(pwd)' -w '$(pwd)' -u '$(uid):$(gid)' \
	  rocstreaming/env-sphinx \
	  make docs

.PHONY: docs-diagrams
docs-diagrams: $(images_svg)

.PHONY: docs-openapi
docs-openapi: rest/openapi.html

.PHONY: docs-serve
docs-serve: docs-diagrams
	while :; do \
	  ( find docs ; echo mkdocs.yml ) | entr -drs \
	    '$(MAKE) --no-print-directory -i docs-diagrams && \
	    mkdocs serve --no-livereload'; \
	  pkill -s 0 mkdocs; \
	done

# Diagrams: generate SVG images from d2 sources.
#
# '--layout dagre' is used by default; skipping it allows to set layout on a
# per-diagram level.
# Other options: '--sketch', '--layout elk'.
%.svg : %.d2
	d2 --theme 0 --dark-theme 200 --pad 5 --scale 0.98 $<

#---------- cleaning ----------#

.PHONY: clean
clean:
	rm -rf $(temp_files)

.PHONY: clean-diagrams
clean-diagrams:
	rm -f $(temp_images)

.PHONY: clean-all
clean-all: clean
	rm -rf $(temp_dirs_aux)
