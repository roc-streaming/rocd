// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package devices

import (
	"crypto/sha1"
	"encoding/hex"
	"fmt"
	"sync"

	"github.com/denisbrodbeck/machineid"
)

var (
	machine     string
	machineOnce sync.Once
)

func makeDeviceUID(systemName string) string {
	// Ensure that machine ID won't change while rocd is running.
	// (Normally it never changes at all, but it's an external
	// resource so we can't be sure).
	machineOnce.Do(func() {
		var err error
		machine, err = machineid.ID()
		if err != nil {
			log.Fatalf("failed to get machine id: %s", err.Error())
		}
	})

	hasher := sha1.New()
	fmt.Fprintf(hasher, "machine=%s,device=%s",
		machine, systemName)

	hashsum := hex.EncodeToString(hasher.Sum(nil))
	uid := ""
	for i := 0; i < len(hashsum); i++ {
		if i > 0 && i%8 == 0 {
			uid += "-"
		}
		uid += string(hashsum[i])
	}

	return uid
}
