// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package streams

import (
	"github.com/roc-streaming/rocd/daemon/devices"
	"github.com/roc-streaming/rocd/daemon/models"
	"github.com/roc-streaming/rocd/daemon/store"
)

type StreamManager struct {
	store         *store.PersistStore
	deviceManager *devices.DeviceManager
}

func NewStreamManager(
	store *store.PersistStore, deviceManager *devices.DeviceManager,
) (*StreamManager, error) {
	log.Infof("initializing streams")

	m := &StreamManager{
		store:         store,
		deviceManager: deviceManager,
	}

	return m, nil
}

func (sm *StreamManager) ListStreams() ([]*models.Stream, error) {
	return nil, nil
}

func (sm *StreamManager) GetStream(uid string) (*models.Stream, error) {
	return nil, nil
}

func (sm *StreamManager) UpdateStream(uid string, stream *models.Stream) (*models.Stream, error) {
	return nil, nil
}

func (sm *StreamManager) CreateStream(stream *models.Stream) (*models.Stream, error) {
	return nil, nil
}

func (sm *StreamManager) DeleteStream(uid string) error {
	return nil
}
