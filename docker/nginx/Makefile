
TAG = 1.13.10
NAME = nginmesh/nginx-dev


all:    build push

build:
	docker build -t $(NAME):$(TAG) --build-arg NGINX_VER=${TAG} .

push:
	docker push $(NAME):$(TAG)

