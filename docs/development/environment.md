# Developer setup

## Build tools

*TODO*

## Documentation tools

### Docker image

There is a [docker image](https://hub.docker.com/repository/docker/rocstreaming/env-docs) that has all tools listed below pre-installed. It is built and published automatically with [github actions](https://github.com/roc-streaming/dockerfiles).

This script will automatically pull the image and build documentation in container:

```
./script/generate_docs.py build
```

It can also start preview server on localhost with automatic rebuild on change:

```
./script/generate_docs.py serve
```

### Manual install

Alternatively, you can install all necessary tools manually.

* [**mkdocs**](https://www.mkdocs.org/) - generates HTML documentation from markdown

    Install Python and pip, then run:

    ```
    pip install mkdocs mkdocs-material mkdocs-material-extensions pymdown-extensions
    ```

    You can either use virtualenv, or pass `--break-system-packages` to pip command and add `~/.local/bin` to PATH.

* [**d2**](https://d2lang.com/) - generates SVG diagrams from text description

    Install Go, then run:

    ```
    go install oss.terrastruct.com/d2@latest
    ```

    Add `~/go/bin` or `$GOPATH/bin` to PATH.

* [**redoc**](https://github.com/Redocly/redoc) - generates HTML reference from openapi.json

    Install node.js and npm, then run:

    ```
    npm install -g @redocly/cli
    ```

    Add `~/.nodejs/bin` to PATH.
