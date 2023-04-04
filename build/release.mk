target/dist:
	$Q mkdir -p target/dist

.PHONY: changelog
.ONESHELL: changelog
changelog: ## Outputs the changes since the last version committed
	$Q VERSION="$(VERSION)" $(CURDIR)/build/changelog.sh

.ONESHELL: target/dist/release_notes.md
target/dist/release_notes.md: target/dist
	$(info $(M) building release notes) @
	$Q echo "# Release Notes" > target/dist/release_notes.md
	VERSION="$(VERSION)" $(CURDIR)/build/changelog.sh >> target/dist/release_notes.md

.PHONY: release-notes
release-notes: target/dist/release_notes.md ## Build release notes

.PHONY: version
version: ## Outputs the current version
	$Q echo "Version: $(VERSION)"

.PHONY: version-update
.ONESHELL: version-update
version-update: ## Prompts for a new version
	$(info $(M) updating repository to new version) @
	$Q echo "  last committed version: $(LAST_VERSION)"
	$Q echo "  Cargo.toml file version : $(VERSION)"
	read -p "  Enter new version in the format (MAJOR.MINOR.PATCH): " version
	$Q echo "$$version" | $(GREP) -qE '^[0-9]+\.[0-9]+\.[0-9]+-?.*$$' || \
		(echo "invalid version identifier: $$version" && exit 1) && \
	$(SED) -i "s/^version\s*=.*$$/version = \"$$version\"/" $(CURDIR)/Cargo.toml
	$(SED) -i "s/^version\s*=.*$$/version = \"$$version\"/" $(CURDIR)/nginx-sys/Cargo.toml
	@ VERSION=$(shell $(GREP) -Po '^version\s+=\s+"\K.*?(?=")' $(CURDIR)/Cargo.toml)

.PHONY: version-release
.ONESHELL: version-release
version-release: ## Change from a pre-release to full release version
	$Q echo "$(VERSION)" | $(GREP) -qE '^[0-9]+\.[0-9]+\.[0-9]+-beta$$' || \
		(echo "invalid version identifier - must contain suffix -beta: $(VERSION)" && exit 1)
	export NEW_VERSION="$(shell echo $(VERSION) | $(SED) -e 's/-beta$$//')"
	$(SED) -i "s/^version\s*=.*$$/version = \"$$version\"/" $(CURDIR)/Cargo.toml
	$(SED) -i "s/^version\s*=.*$$/version = \"$$version\"/" $(CURDIR)/nginx-sys/Cargo.toml
	@ VERSION=$(shell $(GREP) -Po '^version\s+=\s+"\K.*?(?=")' $(CURDIR)/Cargo.toml)

.PHONY: cargo-release
cargo-release: ## Releases a new version to crates.io
	$(info $(M) releasing version $(VERSION) to crates.io) @
	$Q $(CARGO) publish