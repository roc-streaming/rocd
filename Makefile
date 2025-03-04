images_src := $(wildcard docs/assets/dia/*.d2)
images_svg := $(patsubst %.d2,%.svg,$(images_src))

temp_images := \
  docs/assets/dia/*.png \
  docs/assets/dia/*.svg
temp_dirs := \
  site/ \
  target/
temp_dirs_aux := \
  .cache/


default: help

.PHONY: help
help:
	@echo >&2 "Targets:"
	@echo >&2
	@echo >&2 "  docs  - build docs"
	@echo >&2 "  clean - clean artifacts"
	@echo >&2 "  ..." # TODO

#---------- documentation ----------#

# chmod is a fix for mkdocs_puml plugin on Nixos: site/assets has u-w permission
# possible reason:
# - https://github.com/MikhailKravets/mkdocs_puml/blob/2.3.0/mkdocs_puml/plugin.py#L222
# - see `stat /nix/store/*-mkdocs_puml-*/lib/python*/site-packages/mkdocs_puml/static/`
# TODO: report to upstream
.PHONY: docs
docs: docs-diagrams
	mkdocs build --no-directory-urls
	#if [ -d site ]; then chmod -R u+w site ; fi

.PHONY: docs-diagrams
docs-diagrams: $(images_svg)

# Diagrams: generate SVG images from d2 sources.
#
# '--layout dagre' is used by default; skipping it allows to set layout on a
# per-diagram level.
# Other options: '--sketch', '--layout elk'.
%.svg : %.d2
	d2 --dark-theme 200 --pad 5 --scale 0.98 $<

#---------- cleaning ----------#

.PHONY: clean
clean: clean-diagrams
	rm -rf $(temp_dirs)

.PHONY: clean-diagrams
clean-diagrams:
	rm -f $(temp_images)

.PHONY: clean-all
clean-all: clean
	rm -rf $(temp_dirs_aux)
