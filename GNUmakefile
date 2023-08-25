.DEFAULT_GOAL     := help
MAKE_MAJOR_VER    := $(shell echo $(MAKE_VERSION) | cut -d'.' -f1)

ifneq ($(shell test $(MAKE_MAJOR_VER) -gt 3; echo $$?),0)
$(error Make version $(MAKE_VERSION) is not supported, please install GNU Make 4.x)
endif

CACHE_DIR          ?= $(abspath $(CURDIR)/.cache)
GREP               ?= $(shell command -v ggrep 2> /dev/null || command -v grep 2> /dev/null)
SED                ?= $(shell command -v gsed 2> /dev/null || command -v sed 2> /dev/null)
AWK                ?= $(shell command -v gawk 2> /dev/null || command -v awk 2> /dev/null)
VERSION            ?= $(shell $(GREP) -Po '^version\s+=\s+"\K.*?(?=")' $(CURDIR)/Cargo.toml)
CARGO              ?= cargo
DOCKER             ?= docker
DOCKER_BUILD_FLAGS ?= --load
COMMITSAR_DOCKER   := $(DOCKER) run --tty --rm --workdir /src -v "$(CURDIR):/src" aevea/commitsar
COMMITSAR		   ?= $(shell command -v commitsar 2> /dev/null)
PROJECT_NAME       ?= ngx-rust
GITHUB_REPOSITORY  ?= nginxinc/$(PROJECT_NAME)
SRC_REPO           := https://github.com/$(GITHUB_REPOSITORY)

RELEASE_BUILD_FLAGS ?= --quiet --release

Q = $(if $(filter 1,$V),,@)
M = $(shell printf "\033[34;1m▶\033[0m")

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
	FEATURES += --features=linux
endif

# Use docker based commitsar if it isn't in the path
ifeq ($(COMMITSAR),)
	COMMITSAR = $(COMMITSAR_DOCKER)
endif

.PHONY: help
help:
	@$(GREP) --no-filename -E '^[ a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		$(AWK) 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-28s\033[0m %s\n", $$1, $$2}' | sort

.PHONY: clean
clean: clean-cache; $(info $(M) cleaning...)	@ ## Cleanup everything
	$Q $(CARGO) clean

.PHONY: clean-cache
clean-cache: ## Remove all cached dependencies and build artifacts
	$(Q) rm -rf $(CACHE_DIR)

.PHONY: commitsar
commitsar: ## Run git commit linter
	$Q $(info $(M) running commitsar...)
	$(COMMITSAR)

target:
	$Q mkdir -p $@

.PHONY: debug
debug: target/debug ## Build current platform target in debug mode

target/debug:
	$Q echo "$(M) building in debug mode for the current platform"
	$Q $(CARGO) build --quiet

.PHONY: release
release: target/release ## Build current platform target in release mode

target/release:
	$Q echo "$(M) building in release mode for the current platform"
	$Q $(CARGO) build $(RELEASE_BUILD_FLAGS)

.PHONY: test
test: ## Run tests
	$Q $(CARGO) test

.PHONY: format
format: ## Run rustfmt
	$Q $(CARGO) fmt

.PHONY: lint
lint: ## Run clippy
	$Q $(CARGO) clippy

.PHONY: examples-debug
examples-debug:
	$Q echo "$(M) building all examples as debug"
	$Q $(CARGO) build --quiet --package=examples --examples $(FEATURES)

include $(CURDIR)/build/container.mk
include $(CURDIR)/build/release.mk
include $(CURDIR)/build/github.mk
