
TAG = 1.20
NAME = nginmesh/ngx-rust-tool


all:    build push

build:
	docker build -t $(NAME):$(TAG) .

push:
	docker push $(NAME):$(TAG)

