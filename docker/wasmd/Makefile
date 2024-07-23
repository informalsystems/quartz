WASMD_VERSION?=v0.44.0
ORG?=informaldev
IMAGE?=$(ORG)/wasmd
.DEFAULT_GOAL := build

build:
	docker build \
		--build-arg="WASMD_VERSION=$(WASMD_VERSION)" \
		-t $(IMAGE):$(WASMD_VERSION) \
		.
.PHONY: build

run: build
	docker run --rm -it \
		-p 26657:26657 -p 26656:26656 -p 1317:1317 -p 9090:9090 \
		--mount type=volume,source=wasmd_data,target=/root \
		--name wasmd \
		$(ORG)/wasmd:$(WASMD_VERSION)
.PHONY: run

create-accounts: check-wasmd
	mkdir -p ./accounts
	wasmd keys add admin > ./accounts/admin.txt 2>&1
	wasmd keys add alice > ./accounts/alice.txt 2>&1
	wasmd keys add bob > ./accounts/bob.txt 2>&1
	wasmd keys add charlie > ./accounts/charlie.txt 2>&1
.PHONY: create-accounts

delete-accounts: check-wasmd
	wasmd keys delete -y admin
	wasmd keys delete -y alice
	wasmd keys delete -y bob
	wasmd keys delete -y charlie
.PHONY: delete-accounts

check-wasmd:
	# TODO: Check that wasmd version is as expected
.PHONY: check-wasmd