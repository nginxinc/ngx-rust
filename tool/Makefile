
TAG = 1.19.0
NAME = gcr.io/$(GCLOUD_PROJECT)/ngx-rust-tool


all:    build push

build:
	docker build -t $(NAME):$(TAG) .

push:
	gcloud docker --server=gcr.io -- push $(NAME):$(TAG) 

