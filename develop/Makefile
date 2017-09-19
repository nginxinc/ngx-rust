PREFIX=nginmesh/nginx
TAG=1.13.5

build:
	echo 'building nginx image for oss ${PREFIX}:${TAG}'
	docker build -t $(PREFIX):$(TAG) .


push:
	echo 'pushing nginx image to dockerhub ${PREFIX}:${TAG}'
	docker push $(PREFIX):$(TAG)
