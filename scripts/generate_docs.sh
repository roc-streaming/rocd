#! /usr/bin/env bash

# This script is called from roc-streaming/roc-streaming.github.io CI
# so that it doesn't need to know details about pixi & task.

cd "$(dirname "$0")"/..
pixi run task docs
