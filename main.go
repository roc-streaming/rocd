// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package main

//	@title			rocd
//	@version		0.0.1
//	@description	Real-time audio streaming daemon.

//	@license.name	MPL-2.0
//	@license.url	http://mozilla.org/MPL/2.0/

//	@tag.name			Devices
//	@tag.description	Device API
//	@tag.name			Streams
//	@tag.description	Stream API
//	@tag.name			Stream devices
//	@tag.description	Stream device API
//	@tag.name			Monitoring
//	@tag.description	Monitoring API

import (
	"github.com/op/go-logging"

	"github.com/roc-streaming/rocd/daemon/devices"
	"github.com/roc-streaming/rocd/daemon/events"
	_ "github.com/roc-streaming/rocd/daemon/logs"
	"github.com/roc-streaming/rocd/daemon/server"
	"github.com/roc-streaming/rocd/daemon/store"
	"github.com/roc-streaming/rocd/daemon/streams"
)

var log = logging.MustGetLogger("rocd")

func main() {
	log.Noticef("initializing daemon")

	persistStore, err := store.NewPersistStore()
	if err != nil {
		log.Fatal(err)
	}

	eventDispatcher, err := events.NewEventDispatcher()
	if err != nil {
		log.Fatal(err)
	}

	deviceManager, err := devices.NewDeviceManager(persistStore, eventDispatcher)
	if err != nil {
		log.Fatal(err)
	}

	streamManager, err := streams.NewStreamManager(persistStore, deviceManager)
	if err != nil {
		log.Fatal(err)
	}

	srv := server.New(server.Config{
		DeviceManager:   deviceManager,
		StreamManager:   streamManager,
		EventDispatcher: eventDispatcher,
	})

	err = srv.Serve("0.0.0.0:3000")
	if err != nil {
		log.Fatal(err)
	}
}
