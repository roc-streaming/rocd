// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package admin

import (
	"embed"
	"io/fs"
)

//go:embed all:dist
var staticFiles embed.FS

func FileSystem() fs.FS {
	fsys, err := fs.Sub(staticFiles, "dist")
	if err != nil {
		panic(err)
	}

	return fsys
}
