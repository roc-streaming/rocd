docker := docker run \
	-t --rm -v "`pwd`:`pwd`" -w "`pwd`" -u "$(UID)"

all: build

re: fmt gen tidy build

build:
	go build

arm32:
	GOOS=linux GOARCH=arm go build

arm64:
	GOOS=linux GOARCH=arm64 go build

gen:
	go generate ./...
	swag init

fmt:
	go fmt ./...
	swag fmt

tidy:
	go mod tidy

.PHONY: admin
admin:
	cd admin && pnpm --silent run lint
	cd admin && pnpm --silent run build | cat
	go build -v

jsfmt:
	cd admin && pnpm --silent run format

toc:
	markdown-toc --maxdepth 3 -i USAGE.md
	markdown-toc --maxdepth 3 -i HACKING.md

html:
	rm -rf docs/html
	mkdir -p docs/html
	cp docs/*.html docs/html/
	cp docs/*.json docs/html/
	cp docs/*.yaml docs/html/
