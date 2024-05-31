// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package devices

import (
	"os/exec"
	"strings"
)

func command(name string, arg ...string) *exec.Cmd {
	log.Debugf("running command: %s",
		strings.Join(append([]string{name}, arg...), " "))

	return exec.Command(name, arg...)
}
