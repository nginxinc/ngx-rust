
TAG = 1.26.0
NAME = nginxinc/ngx-rust-tool


all:    build push

build:
	docker build -t $(NAME):$(TAG) --build-arg RUST_VERSION=$(TAG) .

push:
	docker push $(NAME):$(TAG)

