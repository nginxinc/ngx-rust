
TAG = 1.25.0
NAME = nginmesh/ngx-rust-tool


all:    build push

build:
	docker build -t $(NAME):$(TAG) --build-arg RUST_VERSION=$(TAG) .

push:
	docker push $(NAME):$(TAG)

