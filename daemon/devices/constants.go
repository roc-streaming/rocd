// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package devices

import "time"

const (
	// retry errors when fetching device list from backend
	fetchRetryCount    = 5
	fetchRetryInterval = 20 * time.Millisecond

	// how long to wait until reconnecting to backend
	backendReconnectInterval = 3 * time.Second
	// how long to wait until backend handles request
	backendResponseTimeout = 5 * time.Second
	// how often to ping backend during startup
	backendPingInterval = 20 * time.Millisecond

	// rate-limit backend events
	eventMinInterval = 20 * time.Millisecond
	// re-read devices periodically
	eventCheckInterval = 15 * time.Second
)

type deviceDir int

const (
	outputDir deviceDir = iota
	inputDir
)

type deviceFilter int

const (
	anyDevice deviceFilter = iota
	streamDevice
)
