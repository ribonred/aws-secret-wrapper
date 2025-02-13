IMAGE_NAME := ghcr.io/ribonred/s3-secret-wrapper
TAG := latest

.PHONY: build push

build:
	docker build -t $(IMAGE_NAME):$(TAG) .

push:
	docker tag $(IMAGE_NAME):$(TAG) $(IMAGE_NAME):latest
	docker push $(IMAGE_NAME):$(TAG)
	docker push $(IMAGE_NAME):latest
