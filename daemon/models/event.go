// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

type EventType string // @name EventType

const (
	EventDeviceAdded   EventType = "device_added"
	EventDeviceRemoved EventType = "device_removed"
	EventDeviceUpdated EventType = "device_updated"
)

type Event struct {
	Type      EventType `json:"type"`
	DeviceUID string    `json:"device_uid,omitempty"`
} // @name Stream

func (e *Event) Dump() string {
	return dump(e)
}

func (e *Event) Compare(eo *Event) int {
	return deriveCompareEvent(e, eo)
}

func (e *Event) Equal(eo *Event) bool {
	return deriveEqualEvent(e, eo)
}

func (e *Event) Hash() uint64 {
	return deriveHashEvent(e)
}

func (e *Event) Clone() *Event {
	return deriveCloneEvent(e)
}
