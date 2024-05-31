// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

import (
	"strings"

	"gopkg.in/yaml.v3"
)

func dump(value any) string {
	b, err := yaml.Marshal(value)
	if err != nil {
		panic(err)
	}

	str := string(b)
	str = strings.TrimSuffix(str, "\n")

	lines := strings.Split(str, "\n")
	for n := range lines {
		lines[n] = "  " + lines[n]
	}

	return strings.Join(lines, "\n")
}
