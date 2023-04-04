.PHONY: container-debian-build-image
.ONESHELL: container-debian-build-image
container-debian-build-image:
container-debian-build-image: ## Builds a container image for building on Debian Linux
	$Q echo "$(M) building debian linux docker build image: $(@)"
	$(DOCKER) buildx build $(DOCKER_BUILD_FLAGS) -t debian-ngx-rust-builder -f Containerfile.debian $(CURDIR);

.PHONY: container-test
container-test: container-debian-build-image ## Run tests inside container
	$Q  mkdir -p .cache/cargo nginx-sys/.nginx
	$(DOCKER) run --rm --volume "$(CURDIR):/project" --workdir /project --env 'CARGO_HOME=/project/.cache/cargo' debian-ngx-rust-builder make test
	# Reset permissions on the target directory to the current user
	if command -v id > /dev/null; then \
		$(DOCKER) run --rm --volume "$(CURDIR):/project" --workdir /project debian-ngx-rust-builder chown --silent --recursive "$(shell id -u):$(shell id -g)" /project/target /project/.cache /project/nginx-sys/.nginx
	fi

.PHONY: container-shell
container-shell: container-debian-build-image ## Start a shell inside container
	$Q  mkdir -p .cache/cargo nginx-sys/.nginx
	$(DOCKER) run -it --rm --volume "$(CURDIR):/project" --workdir /project --env 'CARGO_HOME=/project/.cache/cargo' debian-ngx-rust-builder bash
	# Reset permissions on the target directory to the current user
	if command -v id > /dev/null; then \
		$(DOCKER) run --rm --volume "$(CURDIR):/project" --workdir /project debian-ngx-rust-builder chown --silent --recursive "$(shell id -u):$(shell id -g)" /project/target /project/.cache /project/nginx-sys/.nginx
	fi

.PHONY: build-docker
build-docker: ## build docker image with all example modules
	$(DOCKER) buildx build $(DOCKER_BUILD_FLAGS) -t $(PROJECT_NAME) .
