// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package devices

import (
	"context"
	"errors"
	"fmt"
	"math/rand"
	"sort"
	"sync"
	"time"

	"golang.org/x/time/rate"

	"github.com/roc-streaming/rocd/daemon/events"
	"github.com/roc-streaming/rocd/daemon/models"
	"github.com/roc-streaming/rocd/daemon/store"
)

type DeviceManager struct {
	mu sync.Mutex

	// Device structs are immutable.
	// When devices is modified, a new struct is allocated.
	// Maps themselves are mutable.
	deviceByUID  map[string]*models.Device
	deviceByName map[string]*models.Device

	backend backend
	store   *store.PersistStore
	edisp   *events.EventDispatcher

	baseIndex uint64
	lastIndex uint64
}

func NewDeviceManager(
	store *store.PersistStore, edisp *events.EventDispatcher,
) (*DeviceManager, error) {
	log.Infof("initializing devices")

	backend, err := openBackend(store)
	if err != nil {
		return nil, err
	}

	dm := &DeviceManager{
		backend: backend,
		store:   store,
		edisp:   edisp,
	}

	dm.baseIndex = uint64(rand.Intn(9999))
	dm.lastIndex = 1

	if err := dm.migrateDevices(); err != nil {
		return nil, err
	}

	go dm.listenEvents()

	return dm, nil
}

func (dm *DeviceManager) ListDevices() ([]*models.Device, error) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	return dm.listDevices(anyDevice)
}

func (dm *DeviceManager) GetDevice(uid string) (*models.Device, error) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	return dm.getDevice(uid, anyDevice)
}

func (dm *DeviceManager) UpdateDevice(uid string, device *models.Device) (*models.Device, error) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if device == nil {
		panic("nil device")
	}

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	if err := dm.updateDevice(uid, device, anyDevice); err != nil {
		log.Error(err)
		return nil, err
	}

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	dev, err := dm.getDevice(uid, anyDevice)
	if err != nil {
		log.Error(err)
		return nil, err
	}

	return dev, nil
}

func (dm *DeviceManager) ListStreamDevices() ([]*models.Device, error) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	return dm.listDevices(streamDevice)
}

func (dm *DeviceManager) GetStreamDevice(uid string) (*models.Device, error) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	return dm.getDevice(uid, streamDevice)
}

func (dm *DeviceManager) UpdateStreamDevice(
	uid string, device *models.Device,
) (*models.Device, error) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if device == nil {
		panic("nil device")
	}

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	if err := dm.updateDevice(uid, device, streamDevice); err != nil {
		log.Error(err)
		return nil, err
	}

	dev, err := dm.getDevice(uid, streamDevice)
	if err != nil {
		log.Error(err)
		return nil, err
	}

	return dev, nil
}

func (dm *DeviceManager) CreateStreamDevice(device *models.Device) (*models.Device, error) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if device == nil {
		panic("nil device")
	}

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	uid, err := dm.createStreamDevice(device)
	if err != nil {
		log.Error(err)
		return nil, err
	}

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return nil, err
	}

	dev, err := dm.getDevice(uid, streamDevice)
	if err != nil {
		log.Error(err)
		return nil, err
	}

	return dev, nil
}

func (dm *DeviceManager) DeleteStreamDevice(uid string) error {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if err := dm.rereadDevices(); err != nil {
		log.Error(err)
		return err
	}

	err := dm.deleteStreamDevice(uid)
	if err != nil {
		log.Error(err)
		return err
	}

	return nil
}

func (dm *DeviceManager) listDevices(filter deviceFilter) ([]*models.Device, error) {
	devices := make([]*models.Device, 0, len(dm.deviceByUID))

	for _, dev := range dm.deviceByUID {
		if filter == streamDevice && !dev.IsStream {
			continue
		}
		devices = append(devices, dev)
	}

	sort.Slice(devices, func(i, j int) bool {
		return devices[i].Compare(devices[j]) < 0
	})

	return devices, nil
}

func (dm *DeviceManager) getDevice(uid string, filter deviceFilter) (*models.Device, error) {
	dev, ok := dm.deviceByUID[uid]
	if !ok {
		return nil, fmt.Errorf("device %q not found", uid)
	}
	if filter == streamDevice && !dev.IsStream {
		return nil, errors.New("not a stream device")
	}

	return dev, nil
}

func (dm *DeviceManager) updateDevice(uid string, updDev *models.Device, filter deviceFilter) error {
	log.Debugf("updating device %q", uid)

	dev, ok := dm.deviceByUID[uid]
	if !ok {
		return fmt.Errorf("device %q not found", uid)
	}
	if filter == streamDevice && !dev.IsStream {
		return errors.New("not a stream device")
	}

	// Don't modify existing struct.
	dev = dev.Clone()

	// Refuse to update immutable fields.
	// We don't check some fields like driver and flags that are not likely to
	// be provided intentionally, to be more liberal to external input.
	if updDev.UID != "" && updDev.UID != dev.UID {
		return fmt.Errorf("'device_uid' should be same or empty")
	}
	if updDev.SystemName != "" && updDev.SystemName != dev.SystemName {
		return fmt.Errorf("'system_name' should be same or empty")
	}
	if updDev.DisplayName != "" && updDev.DisplayName != dev.DisplayName {
		return fmt.Errorf("'display_name' should be same or empty")
	}
	if updDev.Type != "" && updDev.Type != dev.Type {
		return fmt.Errorf("'type' should be same or empty")
	}

	// Validate fields.
	if dev.Status != "" && dev.Status != models.StatusDisabled &&
		dev.Status != models.StatusEnabled && dev.Status != models.StatusUnavailable {
		return fmt.Errorf("'status' should be %q, %q, or empty",
			models.StatusEnabled, models.StatusDisabled)
	}

	if dev.IsStream {
		switch dev.Type {
		case models.Sink:
			if err := updDev.ToAddress.Validate(); err != nil {
				return fmt.Errorf("invalid 'to_address': %w", err)
			}
			if updDev.FromAddress.Len() != 0 {
				return fmt.Errorf(
					"with 'type' %q, use 'to_address', not 'from_address", dev.Type)
			}
		case models.Source:
			if err := updDev.FromAddress.Validate(); err != nil {
				return fmt.Errorf("invalid 'from_address': %w", err)
			}
			if updDev.ToAddress.Len() != 0 {
				return fmt.Errorf(
					"with 'type' %q, use 'from_address', not 'to_address", dev.Type)
			}
		}
	} else {
		if updDev.Status != "" && updDev.Status != dev.Status {
			return errors.New(
				"only stream devices can have 'status'")
		}
		if updDev.FromAddress.Len() != 0 || updDev.ToAddress.Len() != 0 {
			return errors.New(
				"only stream device can have 'from_address' or 'to_address'")
		}
	}

	// Apply changes.
	if (updDev.Status == models.StatusDisabled && dev.Status != models.StatusDisabled) ||
		(updDev.Status == models.StatusEnabled && dev.Status != models.StatusEnabled) ||
		(updDev.Status == models.StatusUnavailable && dev.Status == models.StatusDisabled) {
		if updDev.GetEnabled() {
			dev.Status = models.StatusEnabled
		} else {
			dev.Status = models.StatusDisabled
		}
		if err := dm.backend.applyEnabled(dev); err != nil {
			return err
		}
	}
	if updDev.IsMuted != nil && updDev.GetMuted() != dev.GetMuted() {
		dev.SetMuted(updDev.GetMuted())
		if err := dm.backend.applyMuted(dev); err != nil {
			return err
		}
	}
	if updDev.ToAddress != nil && !updDev.ToAddress.Equal(dev.ToAddress) {
		dev.ToAddress = updDev.ToAddress.Clone()
		if err := dm.backend.applyAddress(dev); err != nil {
			return err
		}
	}
	if updDev.FromAddress != nil && !updDev.FromAddress.Equal(dev.FromAddress) {
		dev.FromAddress = updDev.FromAddress.Clone()
		if err := dm.backend.applyAddress(dev); err != nil {
			return err
		}
	}

	// Stream devices are stored persistently.
	if dev.IsStream {
		err := dm.store.SaveStreamDevice(dev)
		if err != nil {
			return fmt.Errorf("failed to save device: %w", err)
		}
	}

	// Commit changes.
	dm.deviceByUID[uid] = dev
	dm.deviceByName[uid] = dev

	return nil
}

func (dm *DeviceManager) createStreamDevice(dev *models.Device) (string, error) {
	log.Debugf("creating new virtual device")

	// Ensure we're the only owner of the struct.
	dev = dev.Clone()

	// Refuse to set fields that we're going to generate by ourselves.
	// We don't check some fields like driver and flags that are not likely to
	// be provided intentionally, to be more liberal to external input.
	if dev.UID != "" {
		return "", fmt.Errorf("'device_uid' should be empty")
	}

	// Validate fields.
	if dev.Type != models.Sink && dev.Type != models.Source {
		return "", fmt.Errorf("'type' should be %q or %q", models.Sink, models.Source)
	}
	if dev.Status != "" && dev.Status != models.StatusDisabled &&
		dev.Status != models.StatusEnabled && dev.Status != models.StatusUnavailable {
		return "", fmt.Errorf("'status' should be %q, %q, or empty",
			models.StatusEnabled, models.StatusDisabled)
	}

	switch dev.Type {
	case models.Sink:
		if err := dev.ToAddress.Validate(); err != nil {
			return "", fmt.Errorf("invalid 'to_address': %w", err)
		}
		if dev.FromAddress.Len() != 0 {
			return "", fmt.Errorf(
				"with 'type' %q, use 'to_address', not 'from_address", dev.Type)
		}
	case models.Source:
		if err := dev.FromAddress.Validate(); err != nil {
			return "", fmt.Errorf("invalid 'from_address': %w", err)
		}
		if dev.ToAddress.Len() != 0 {
			return "", fmt.Errorf(
				"with 'type' %q, use 'from_address', not 'to_address", dev.Type)
		}
	}

	// Set defaults.
	if dev.SystemName != "" {
		_, ok := dm.deviceByName[dev.SystemName]
		if ok {
			return "", fmt.Errorf("'system_name' %q already exists", dev.SystemName)
		}
	} else {
		dev.SystemName = dm.generateDeviceName(dev.Type)
	}

	if dev.DisplayName == "" {
		if dev.Type == models.Sink {
			dev.DisplayName = fmt.Sprintf("Roc Sender")
		} else {
			dev.DisplayName = fmt.Sprintf("Roc Receiver")
		}
	}

	if dev.Status == "" || dev.Status == models.StatusUnavailable {
		dev.Status = models.StatusEnabled
	}

	dev.UID = makeDeviceUID(dev.SystemName)
	dev.Driver = dm.backend.driver()

	dev.IsHardware = false
	dev.IsStream = true

	dev.SetDefaults()

	_, ok := dm.deviceByUID[dev.UID]
	if ok {
		// Can't happen?
		return "", fmt.Errorf("device %q already exists", dev.UID)
	}

	// (Re)create device on backend.
	// May update device struct.
	err := dm.backend.resetStreamDevice(dev)
	if err != nil {
		return "", err
	}

	log.Debugf("initialized device:\n%s", dev.Dump())

	// Stream devices are stored persistently.
	err = dm.store.SaveStreamDevice(dev)
	if err != nil {
		return "", fmt.Errorf("failed to save device: %w", err)
	}

	// Commit changes.
	dm.deviceByUID[dev.UID] = dev
	dm.deviceByName[dev.SystemName] = dev

	return dev.UID, nil
}

func (dm *DeviceManager) deleteStreamDevice(uid string) error {
	log.Debugf("deleting virtual device %q", uid)

	hasInStore := dm.store.HasStreamDevice(uid)

	dev, hasOnBacked := dm.deviceByUID[uid]

	if !hasOnBacked && !hasInStore {
		return fmt.Errorf("device %q not found", uid)
	}

	if dev != nil && !dev.IsStream {
		return fmt.Errorf("not a stream device")
	}

	if hasOnBacked {
		log.Debugf("destroying device:\n%s", dev.Dump())

		if err := dm.backend.destroyStreamDevice(dev); err != nil {
			return fmt.Errorf("failed to remove device: %w", err)
		}
	}

	if hasInStore {
		err := dm.store.RemoveStreamDevice(uid)
		if err != nil {
			return fmt.Errorf("failed to remove device: %w", err)
		}
	}

	if dev != nil {
		delete(dm.deviceByUID, dev.UID)
		delete(dm.deviceByName, dev.SystemName)
	}

	return nil
}

// Detect cases when saved device UID became inconsistent with its SystemName.
//
// This can happen if machine ID changed or UID generation algorithm changed.
// Ideally this should never be so, but we still handle it to prevent mess
// in case of unusual scenarios like copying files to another machine.
//
// Since this is called in constructor, the rest of the code may safely assume
// that UID and SystemName are always consistent.
func (dm *DeviceManager) migrateDevices() error {
	for _, savedDev := range dm.store.LoadStreamDevices() {
		recalcUID := makeDeviceUID(savedDev.SystemName)
		if savedDev.UID == recalcUID {
			continue
		}

		log.Warningf("detected uid change, migarting device %q to new uid %q",
			savedDev.UID, recalcUID)

		log.Debugf("migrated device:\n%s", savedDev.Dump())

		if err := dm.store.RemoveStreamDevice(savedDev.UID); err != nil {
			return fmt.Errorf("failed to remove device %q: %w", savedDev.UID, err)
		}

		savedDev = savedDev.Clone()
		savedDev.UID = recalcUID

		if err := dm.store.SaveStreamDevice(savedDev); err != nil {
			return fmt.Errorf("failed to save device %q: %w", savedDev.UID, err)
		}
	}

	return nil
}

// This method re-creates or re-enables saved devices which are present and enabled
// in storage, but are missing or disabled on backend.
//
// It is called in two cases:
//   - when rocd starts
//   - when backend reports that it wiped all devices (e.g. when PipeWire restarts).
func (dm *DeviceManager) restoreDevices() {
	log.Noticef("restoring saved devices")

	nRestored := 0
	nErrors := 0

	for _, savedDev := range dm.store.LoadStreamDevices() {
		if !savedDev.GetEnabled() {
			continue
		}

		oldDev, _ := dm.deviceByUID[savedDev.UID]
		if oldDev != nil && oldDev.Status == models.StatusEnabled {
			continue
		}

		if oldDev == nil {
			// Device is missing from backend, create it.
			log.Debugf("creating device %q", savedDev.UID)

			createDev := savedDev.Clone()
			createDev.UID = ""
			createDev.Status = models.StatusEnabled

			if _, err := dm.createStreamDevice(createDev); err != nil {
				log.Errorf("failed to create device %q: %s", savedDev.UID, err)
				nErrors++
				continue
			}

			if createDev.UID != savedDev.UID {
				panic("unexpected uid change")
			}
			savedDev = createDev
		} else {
			// Device is present on backend, but is disabled, enable it.
			log.Debugf("enabling device %q", savedDev.UID)

			updateDev := savedDev.Clone()
			updateDev.Status = models.StatusEnabled

			if err := dm.updateDevice(updateDev.UID, updateDev, streamDevice); err != nil {
				log.Errorf("failed to enable device %q: %s", savedDev.UID, err)
				nErrors++
				continue
			}

			savedDev = updateDev
		}

		nRestored++

		// Save changes back to storage.
		newDev, _ := dm.deviceByUID[savedDev.UID]
		if newDev != nil && !newDev.Equal(savedDev) {
			if err := dm.store.SaveStreamDevice(newDev); err != nil {
				log.Errorf("failed to save device %q: %s", savedDev.UID, err)
				nErrors++
				continue
			}
		}
	}

	if nErrors == 0 {
		log.Noticef("restored %d device(s), %d error(s)", nRestored, nErrors)
	} else {
		log.Warningf("restored %d device(s), %d error(s)", nRestored, nErrors)
	}
}

// Re-read device list from backend to memory and to storage.
// Update deviceByUID and deviceByName maps.
func (dm *DeviceManager) rereadDevices() error {
	// Collect configured devices from persistent storage.
	savedDevices := make(map[string]*models.Device)
	for _, dev := range dm.store.LoadStreamDevices() {
		dev = dev.Clone()
		dev.SetDefaults()
		savedDevices[dev.UID] = dev
	}

	// Collect actual devices from backend.
	backendDevices := make(map[string]*models.Device)
	for _, dev := range dm.fetchDevices() {
		dev = dev.Clone()
		dev.SetDefaults()
		backendDevices[dev.UID] = dev
	}

	// Propagate changes from backend to storage.
	updatedDevices := make(map[string]*models.Device)
	for _, savedDev := range savedDevices {
		backendDev, _ := backendDevices[savedDev.UID]
		if backendDev == nil || !backendDev.IsStream || backendDev.Equal(savedDev) {
			continue
		}
		if !savedDev.GetEnabled() && backendDev.GetEnabled() {
			log.Warningf("enabling device because it's present on backend")
			log.Debugf("enabling device:\n%s", savedDev.Dump())
		}
		updatedDevices[savedDev.UID] = backendDev
	}
	if len(updatedDevices) != 0 {
		err := dm.store.SaveStreamDevices(updatedDevices)
		if err != nil {
			return fmt.Errorf("failed to save updated devices: %w", err)
		}
	}

	// Rebuild mappings.
	dm.deviceByUID = make(map[string]*models.Device)
	for _, dev := range backendDevices {
		// Add actual backend devices.
		dm.deviceByUID[dev.UID] = dev
	}
	for _, dev := range savedDevices {
		if _, ok := dm.deviceByUID[dev.UID]; !ok {
			// Add saved devices not present on backend.
			if dev.Status == models.StatusEnabled {
				dev.Status = models.StatusUnavailable
			}
			dm.deviceByUID[dev.UID] = dev
		}
	}

	dm.deviceByName = make(map[string]*models.Device)
	for _, dev := range backendDevices {
		dm.deviceByName[dev.SystemName] = dev
	}

	return nil
}

func (dm *DeviceManager) fetchDevices() []*models.Device {
	var devices []*models.Device
	var err error

	for n := 0; n < fetchRetryCount; n++ {
		if n != 0 {
			time.Sleep(fetchRetryInterval)
		}
		devices, err = dm.backend.fetchDevices()
		if err == nil {
			break
		}
	}

	if err != nil {
		log.Warningf("failed to fetch device list: %s", err)
	}

	return devices
}

func (dm *DeviceManager) generateDeviceName(dt models.DeviceType) string {
	for {
		name := fmt.Sprintf("rocd.stream_%s.%v.%v", dt, dm.baseIndex, dm.lastIndex)
		dm.lastIndex++

		if _, ok := dm.deviceByName[name]; ok {
			continue
		}

		used := false
		for _, dev := range dm.store.LoadStreamDevices() {
			if name == dev.SystemName {
				used = true
				break
			}
		}
		if used {
			continue
		}

		return name
	}
}

func (dm *DeviceManager) listenEvents() {
	eventCh := dm.backend.listenDevices()

	ticker := time.NewTicker(eventCheckInterval)
	defer ticker.Stop()

	lim := rate.NewLimiter(rate.Limit(eventMinInterval), 1)

	for {
		var ev backendEvent

		select {
		case ev = <-eventCh:
		case <-ticker.C:
			ev = backendEvent{eventType: eventPeriodicUpdate}
		}

		lim.Wait(context.Background())

		dm.processEvent(ev)
	}
}

func (dm *DeviceManager) processEvent(ev backendEvent) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	if ev.eventType != eventPeriodicUpdate {
		log.Infof("processing event %q", ev.eventType)
	}

	switch ev.eventType {
	case eventDeviceListWiped:
		// Backend lost all configured devices, we should re-create them.
		// This event is generated for backends that don't store devices
		// persistently, e.g. PipeWire.
		if err := dm.rereadDevices(); err != nil {
			log.Error(err)
			return
		}

		dm.restoreDevices()

		if err := dm.rereadDevices(); err != nil {
			log.Error(err)
		}

	case eventDeviceListUpdated, eventPeriodicUpdate:
		if err := dm.rereadDevices(); err != nil {
			log.Error(err)
			return
		}

	case eventDeviceRemoved:
		if err := dm.processRemovedDevice(ev.deviceUID); err != nil {
			log.Error(err)
			return
		}
		if err := dm.rereadDevices(); err != nil {
			log.Error(err)
		}
	}

	// Detect changes and send events to clients.
	dm.edisp.UpdateDevices(dm.deviceByUID)
}

// If user explicitly removed device on backend (using OS-specific tools),
// automatically disable it. We don't want to annoy user and re-create
// device after user tried to remove it. But we also don't want to loss
// device settings, so we disable it instead of deleting. Device can
// be always re-enabled via HTTP API.
func (dm *DeviceManager) processRemovedDevice(uid string) error {
	savedDev, _ := dm.store.LoadStreamDevice(uid)

	if savedDev == nil || !savedDev.GetEnabled() {
		// Nothing to do.
		return nil
	}

	log.Warningf("disabling device because it was removed from backend")

	savedDev = savedDev.Clone()
	savedDev.Status = models.StatusDisabled

	log.Debugf("disabled device:\n%s", savedDev.Dump())

	if err := dm.store.SaveStreamDevice(savedDev); err != nil {
		return err
	}

	return nil
}
