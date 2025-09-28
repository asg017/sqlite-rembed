SHELL := /bin/bash

VERSION=$(shell cat VERSION)

ifeq ($(shell uname -s),Darwin)
CONFIG_DARWIN=y
else ifeq ($(OS),Windows_NT)
CONFIG_WINDOWS=y
else
CONFIG_LINUX=y
endif

LIBRARY_PREFIX=lib
ifdef CONFIG_DARWIN
LOADABLE_EXTENSION=dylib
STATIC_EXTENSION=a
endif

ifdef CONFIG_LINUX
LOADABLE_EXTENSION=so
STATIC_EXTENSION=a
endif


ifdef CONFIG_WINDOWS
LOADABLE_EXTENSION=dll
LIBRARY_PREFIX=
STATIC_EXTENSION=lib
endif

prefix=dist
TARGET_LOADABLE=$(prefix)/debug/rembed0.$(LOADABLE_EXTENSION)
TARGET_LOADABLE_RELEASE=$(prefix)/release/rembed0.$(LOADABLE_EXTENSION)

TARGET_STATIC=$(prefix)/debug/$(LIBRARY_PREFIX)sqlite_rembed0.$(STATIC_EXTENSION)
TARGET_STATIC_RELEASE=$(prefix)/release/$(LIBRARY_PREFIX)sqlite_rembed0.$(STATIC_EXTENSION)

TARGET_H=$(prefix)/debug/sqlite-rembed.h
TARGET_H_RELEASE=$(prefix)/release/sqlite-rembed.h

TARGET_WHEELS=$(prefix)/debug/wheels
TARGET_WHEELS_RELEASE=$(prefix)/release/wheels

PYTHON_PACKAGE_DIR=bindings/python
PYTHON_MODULE_DIR=$(PYTHON_PACKAGE_DIR)/sqlite_rembed
INTERMEDIATE_PYPACKAGE_EXTENSION=$(PYTHON_MODULE_DIR)/rembed0.$(LOADABLE_EXTENSION)

ifdef target
CARGO_TARGET=--target=$(target)
BUILT_LOCATION=target/$(target)/debug/$(LIBRARY_PREFIX)sqlite_rembed.$(LOADABLE_EXTENSION)
BUILT_LOCATION_RELEASE=target/$(target)/release/$(LIBRARY_PREFIX)sqlite_rembed.$(LOADABLE_EXTENSION)
BUILT_LOCATION_STATIC=target/$(target)/debug/$(LIBRARY_PREFIX)sqlite_rembed.$(STATIC_EXTENSION)
BUILT_LOCATION_STATIC_RELEASE=target/$(target)/release/$(LIBRARY_PREFIX)sqlite_rembed.$(STATIC_EXTENSION)
else
CARGO_TARGET=
BUILT_LOCATION=target/debug/$(LIBRARY_PREFIX)sqlite_rembed.$(LOADABLE_EXTENSION)
BUILT_LOCATION_RELEASE=target/release/$(LIBRARY_PREFIX)sqlite_rembed.$(LOADABLE_EXTENSION)
BUILT_LOCATION_STATIC=target/debug/$(LIBRARY_PREFIX)sqlite_rembed.$(STATIC_EXTENSION)
BUILT_LOCATION_STATIC_RELEASE=target/release/$(LIBRARY_PREFIX)sqlite_rembed.$(STATIC_EXTENSION)
endif

ifdef python
PYTHON=$(python)
else
PYTHON=python3
endif

ifdef IS_MACOS_ARM
RENAME_WHEELS_ARGS=--is-macos-arm
else
RENAME_WHEELS_ARGS=
endif

$(prefix):
	mkdir -p $(prefix)/debug
	mkdir -p $(prefix)/release

$(TARGET_WHEELS): $(prefix)
	mkdir -p $(TARGET_WHEELS)

$(TARGET_WHEELS_RELEASE): $(prefix)
	mkdir -p $(TARGET_WHEELS_RELEASE)

$(TARGET_LOADABLE): $(prefix) $(shell find . -type f -name '*.rs')
	cargo build --verbose $(CARGO_TARGET)
	cp $(BUILT_LOCATION) $@

$(TARGET_LOADABLE_RELEASE): $(prefix) $(shell find . -type f -name '*.rs')
	cargo build --verbose --release $(CARGO_TARGET)
	cp $(BUILT_LOCATION_RELEASE) $@

$(TARGET_STATIC): $(prefix) $(shell find . -type f -name '*.rs')
	cargo build --verbose $(CARGO_TARGET) --features=sqlite-loadable/static
	ls target
	ls target/$(target)/debug
	cp $(BUILT_LOCATION_STATIC) $@

$(TARGET_STATIC_RELEASE): $(prefix) $(shell find . -type f -name '*.rs')
	cargo build --verbose --release $(CARGO_TARGET) --features=sqlite-loadable/static
	cp $(BUILT_LOCATION_STATIC_RELEASE) $@

$(TARGET_H): sqlite-rembed.h
	cp $< $@

$(TARGET_H_RELEASE): sqlite-rembed.h
	cp $< $@

Cargo.toml: VERSION
	cargo set-version `cat VERSION`

version:
	make Cargo.toml

format:
	cargo fmt

release: $(TARGET_LOADABLE_RELEASE) $(TARGET_STATIC_RELEASE)

loadable: $(TARGET_LOADABLE)
loadable-release: $(TARGET_LOADABLE_RELEASE)

static: $(TARGET_STATIC) $(TARGET_H)
static-release: $(TARGET_STATIC_RELEASE) $(TARGET_H_RELEASE)

debug: loadable static
release: loadable-release static-release

clean:
	rm dist/*
	cargo clean

test-loadable: loadable
	$(PYTHON) examples/sql/basic.sql

test-python: python
	cd $(PYTHON_PACKAGE_DIR) && $(PYTHON) tests/test_basic.py

# Python packaging targets
python: $(TARGET_LOADABLE)
	mkdir -p $(PYTHON_MODULE_DIR)
	cp $(TARGET_LOADABLE) $(INTERMEDIATE_PYPACKAGE_EXTENSION)
	@echo "✓ Copied extension to Python package"

python-release: $(TARGET_LOADABLE_RELEASE)
	mkdir -p $(PYTHON_MODULE_DIR)
	cp $(TARGET_LOADABLE_RELEASE) $(INTERMEDIATE_PYPACKAGE_EXTENSION)
	@echo "✓ Copied release extension to Python package"

python-wheel: python-release
	cd $(PYTHON_PACKAGE_DIR) && $(PYTHON) -m pip install --upgrade build
	cd $(PYTHON_PACKAGE_DIR) && $(PYTHON) -m build --wheel
	mkdir -p $(TARGET_WHEELS_RELEASE)
	cp $(PYTHON_PACKAGE_DIR)/dist/*.whl $(TARGET_WHEELS_RELEASE)/
	@echo "✓ Built Python wheel in $(TARGET_WHEELS_RELEASE)"

python-sdist: python-release
	cd $(PYTHON_PACKAGE_DIR) && $(PYTHON) -m pip install --upgrade build
	cd $(PYTHON_PACKAGE_DIR) && $(PYTHON) -m build --sdist
	@echo "✓ Built Python source distribution"

python-install: python
	cd $(PYTHON_PACKAGE_DIR) && $(PYTHON) -m pip install -e .
	@echo "✓ Installed Python package in development mode"

python-clean:
	rm -rf $(PYTHON_PACKAGE_DIR)/build
	rm -rf $(PYTHON_PACKAGE_DIR)/dist
	rm -rf $(PYTHON_PACKAGE_DIR)/*.egg-info
	rm -f $(INTERMEDIATE_PYPACKAGE_EXTENSION)

publish-release:
	./scripts/publish_release.sh

.PHONY: clean \
	test test-loadable test-python test-npm test-deno \
	loadable loadable-release \
	static static-release \
	debug release \
	python python-release python-wheel python-sdist python-install python-clean \
	format version publish-release
