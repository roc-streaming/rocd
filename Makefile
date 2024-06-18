.PHONY: all
all: build

.PHONY: re
re: fmt gen tidy build

.PHONY: build
build:
	go build

.PHONY: arm32
arm32:
	GOOS=linux GOARCH=arm go build

.PHONY: arm64
arm64:
	GOOS=linux GOARCH=arm64 go build

.PHONY: gen
gen:
	go generate ./...
	swag init

.PHONY: fmt
fmt:
	go fmt ./...
	swag fmt

.PHONY: tidy
tidy:
	go mod tidy

.PHONY: admin
admin:
	cd admin && pnpm --silent run lint
	cd admin && pnpm --silent run build | cat
	go build -v

.PHONY: jsfmt
jsfmt:
	cd admin && pnpm --silent run format

.PHONY: toc
toc:
	markdown-toc --maxdepth 3 -i USAGE.md
	markdown-toc --maxdepth 3 -i HACKING.md
