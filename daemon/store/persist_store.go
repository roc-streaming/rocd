// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package store

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"os/user"
	"path/filepath"
	"sort"
	"strings"
	"sync"

	"github.com/natefinch/atomic"
	"gopkg.in/yaml.v3"

	"github.com/roc-streaming/rocd/daemon/models"
)

type config struct {
	Devices []*models.Device `yaml:"stream_devices"`
	Streams []*models.Stream `yaml:"streams"`
}

type PersistStore struct {
	mu   sync.RWMutex
	path string

	// Device and Stream structs are immutable.
	// Maps themselves are mutable.
	devices map[string]*models.Device
	streams map[string]*models.Stream
}

func NewPersistStore() (*PersistStore, error) {
	log.Infof("initializing storage")

	ps := &PersistStore{
		path: filepath.Join(defaultDir(), "state.yaml"),
	}

	if err := ps.load(); err != nil {
		return nil, err
	}

	log.Debugf("loaded: %d stream device(s), %d stream(s)",
		len(ps.devices), len(ps.streams))

	return ps, nil
}

func (ps *PersistStore) HasStreamDevice(uid string) bool {
	ps.mu.RLock()
	defer ps.mu.RUnlock()

	dev, _ := ps.devices[uid]
	return dev != nil
}

func (ps *PersistStore) LoadStreamDevices() []*models.Device {
	ps.mu.RLock()
	defer ps.mu.RUnlock()

	devices := make([]*models.Device, 0, len(ps.devices))
	for _, dev := range ps.devices {
		if dev != nil {
			devices = append(devices, dev)
		}
	}

	return devices
}

func (ps *PersistStore) SaveStreamDevices(devices map[string]*models.Device) error {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	if devices == nil {
		panic("nil devices")
	}

	hasChanges := false
	for _, newDev := range devices {
		if newDev == nil {
			panic("nil device")
		}
		if !newDev.IsStream {
			panic("not a stream device")
		}
		oldDev, _ := ps.devices[newDev.UID]
		if oldDev == nil || !oldDev.Equal(newDev) {
			log.Debugf("saving device %q", newDev.UID)
			ps.devices[newDev.UID] = newDev.Clone()
			hasChanges = true
		}
	}

	if !hasChanges {
		return nil
	}

	return ps.save()
}

func (ps *PersistStore) LoadStreamDevice(uid string) (*models.Device, error) {
	ps.mu.RLock()
	defer ps.mu.RUnlock()

	device, _ := ps.devices[uid]
	if device == nil {
		return nil, errors.New("device not found")
	}

	return device, nil
}

func (ps *PersistStore) SaveStreamDevice(device *models.Device) error {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	if device == nil {
		panic("nil device")
	}
	if !device.IsStream {
		panic("not a stream device")
	}

	oldDevice, _ := ps.devices[device.UID]
	if oldDevice != nil && oldDevice.Equal(device) {
		return nil
	}

	log.Debugf("saving device %q", device.UID)

	ps.devices[device.UID] = device.Clone()

	return ps.save()
}

func (ps *PersistStore) RemoveStreamDevice(uid string) error {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	_, ok := ps.devices[uid]
	if !ok {
		return nil
	}

	log.Debugf("removing device %q", uid)

	delete(ps.devices, uid)

	return ps.save()
}

func (ps *PersistStore) HasStream(uid string) bool {
	ps.mu.RLock()
	defer ps.mu.RUnlock()

	srm, _ := ps.streams[uid]
	return srm != nil
}

func (ps *PersistStore) LoadStreams() []*models.Stream {
	ps.mu.RLock()
	defer ps.mu.RUnlock()

	streams := make([]*models.Stream, 0, len(ps.streams))
	for _, stm := range ps.streams {
		if stm != nil {
			streams = append(streams, stm)
		}
	}

	return streams
}

func (ps *PersistStore) SaveStreams(streams map[string]*models.Stream) error {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	if streams == nil {
		panic("nil streams")
	}

	hasChanges := false
	for _, newSrm := range streams {
		if newSrm == nil {
			panic("nil stream")
		}
		oldSrm, _ := ps.streams[newSrm.UID]
		if oldSrm == nil || !oldSrm.Equal(newSrm) {
			log.Debugf("saving stream %q", newSrm.UID)
			ps.streams[newSrm.UID] = newSrm.Clone()
			hasChanges = true
		}
	}

	if !hasChanges {
		return nil
	}

	log.Debugf("saving streams")

	return ps.save()
}

func (ps *PersistStore) LoadStream(uid string) (*models.Stream, error) {
	ps.mu.RLock()
	defer ps.mu.RUnlock()

	stream, _ := ps.streams[uid]
	if stream == nil {
		return nil, errors.New("stream not found")
	}

	return stream, nil
}

func (ps *PersistStore) SaveStream(stream *models.Stream) error {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	if stream == nil {
		panic("nil stream")
	}

	oldStream, _ := ps.streams[stream.UID]
	if oldStream != nil && oldStream.Equal(stream) {
		return nil
	}

	log.Debugf("saving stream %q", stream.UID)

	ps.streams[stream.UID] = stream.Clone()

	return ps.save()
}

func (ps *PersistStore) RemoveStream(uid string) error {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	_, ok := ps.streams[uid]
	if !ok {
		return nil
	}

	log.Debugf("removing stream %q", uid)

	delete(ps.streams, uid)

	return ps.save()
}

func (ps *PersistStore) load() error {
	log.Debugf("loading configuration from %q", ps.path)

	b, err := os.ReadFile(ps.path)

	notExists := err != nil && errors.Is(err, os.ErrNotExist)
	isEmpty := len(b) == 0 || strings.TrimSpace(string(b)) == ""

	var conf config

	switch {
	case notExists:
		log.Debugf("configuration file does not exist")

	case isEmpty:
		log.Debugf("configuration file is empty")

	default:
		err := yaml.Unmarshal(b, &conf)
		if err != nil {
			return fmt.Errorf("failed to load configuration from %q: %w", ps.path, err)
		}
	}

	ps.devices = make(map[string]*models.Device)
	ps.streams = make(map[string]*models.Stream)

	for _, dev := range conf.Devices {
		if dev == nil || dev.UID == "" || !dev.IsStream {
			log.Warningf("ignoring invalid device in yaml file")
			continue
		}
		ps.devices[dev.UID] = dev
	}
	for _, stm := range conf.Streams {
		if stm == nil || stm.UID == "" {
			log.Warningf("ignoring invalid stream in yaml file")
			continue
		}
		ps.streams[stm.UID] = stm
	}

	return nil
}

func (ps *PersistStore) save() error {
	log.Debugf("saving configuration to %q", ps.path)

	defDir := defaultDir()

	if isParentDir(defDir, ps.path) {
		// Default directory is created automatically.
		// If user specified other directory, don't create it.
		err := os.MkdirAll(defDir, 0755)
		if err != nil {
			log.Fatalf("failed to create directory: %s", err.Error())
		}
	}

	conf := config{
		Devices: make([]*models.Device, 0, len(ps.devices)),
		Streams: make([]*models.Stream, 0, len(ps.streams)),
	}

	for _, dev := range ps.devices {
		conf.Devices = append(conf.Devices, dev)
	}
	for _, stm := range ps.streams {
		conf.Streams = append(conf.Streams, stm)
	}

	sort.Slice(conf.Devices, func(i, j int) bool {
		return conf.Devices[i].Compare(conf.Devices[j]) < 0
	})
	sort.Slice(conf.Streams, func(i, j int) bool {
		return conf.Streams[i].Compare(conf.Streams[j]) < 0
	})

	b, err := yaml.Marshal(&conf)
	if err != nil {
		log.Fatalf("failed to marshal yaml: %s", err.Error())
	}

	err = atomic.WriteFile(ps.path, bytes.NewReader(b))
	if err != nil {
		return fmt.Errorf("failed to save configuration to %q: %w", ps.path, err)
	}

	return nil
}

func defaultDir() string {
	usr, err := user.Current()
	if err != nil {
		log.Fatalf("failed to get current user: %v", err.Error())
	}

	return filepath.Join(usr.HomeDir, ".config", "rocd")
}

func isParentDir(parent, child string) bool {
	var err error

	parent, err = filepath.Abs(parent)
	if err != nil {
		return false
	}

	child, err = filepath.Abs(child)
	if err != nil {
		return false
	}

	parent += string(os.PathSeparator)
	return strings.HasPrefix(child, parent)
}
