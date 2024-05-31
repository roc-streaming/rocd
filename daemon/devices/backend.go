// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package devices

import (
	"errors"
	"sort"

	"github.com/roc-streaming/rocd/daemon/models"
	"github.com/roc-streaming/rocd/daemon/store"
)

var (
	// Returned from init().
	// Indicates that this backend in not available on system
	// and should be skipped.
	errNotAvailable = errors.New("not available")
)

// During open, backends are tried from highest to lowest priority.
type backendPrio int

const (
	prioLow backendPrio = iota
	prioMedium
	prioHigh
)

type backendEventType string

const (
	// Driver reported that it lost all configured devices.
	// Device manager should re-create saved devices.
	eventDeviceListWiped backendEventType = "device_list_wiped"

	// Driver reported that some devices were added, updated, or removed.
	// Device manager should rebuild device list.
	eventDeviceListUpdated backendEventType = "device_list_updated"

	// Driver reported that user manually removed device.
	// Device manager should disable device.
	eventDeviceRemoved backendEventType = "device_removed"

	// Periodic update timer ticked.
	// Used to catch events missed by backend.
	eventPeriodicUpdate backendEventType = "periodic_update"
)

type backendEvent struct {
	eventType backendEventType
	deviceUID string
}

// Backend interface used by DeviceManager.
// All methods are invoked under device manager lock.
type backend interface {
	driver() models.DeviceDriver
	prio() backendPrio

	// Try to initialize backend.
	// If returns errNotAvailable, backend is silently skipped.
	// If returns other error, rocd initialization fails.
	init(store *store.PersistStore) error

	// Get channel to receive events from backend.
	listenDevices() <-chan backendEvent
	// Get up-to-date list of devices from backend.
	fetchDevices() ([]*models.Device, error)

	// Create or re-create stream device.
	resetStreamDevice(device *models.Device) error
	// Destroy stream device.
	destroyStreamDevice(device *models.Device) error

	// Apply device IsEnabled property.
	applyEnabled(device *models.Device) error
	// Apply device IsMuted property.
	applyMuted(device *models.Device) error
	// Apply device FromAddress or ToAddress property.
	applyAddress(device *models.Device) error
}

// All backends enabled at compile time register themselves here.
var backends []backend

func openBackend(store *store.PersistStore) (backend, error) {
	sort.Slice(backends, func(i, j int) bool {
		if backends[i].prio() > backends[j].prio() {
			return true
		}
		if backends[i].prio() < backends[j].prio() {
			return false
		}
		return backends[i].driver() < backends[j].driver()
	})

	for _, back := range backends {
		err := back.init(store)
		if err == errNotAvailable {
			continue
		}
		if err != nil {
			return nil, err
		}
		log.Debugf("using %q backend", back.driver())
		return back, nil
	}

	return nil, errors.New("no device backend available")
}
