// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

type DeviceType string // @name DeviceType

const (
	Sink   DeviceType = "sink"
	Source DeviceType = "source"
)

type DeviceStatus string // @name DeviceStatus

const (
	StatusDisabled    DeviceStatus = "disabled"
	StatusEnabled     DeviceStatus = "enabled"
	StatusUnavailable DeviceStatus = "unavailable"
)

type DeviceDriver string // @name DeviceDriver

const (
	DriverPipewire   DeviceDriver = "pipewire"
	DriverPulseaudio DeviceDriver = "pulseaudio"
)

type Device struct {
	// Immutable fields (assigned on creation)

	UID string `json:"device_uid" yaml:"uid"`

	SystemName  string `json:"system_name" yaml:"system_name"`
	DisplayName string `json:"display_name" yaml:"display_name"`

	Type   DeviceType   `json:"type" yaml:"type"`
	Driver DeviceDriver `json:"driver" yaml:"driver"`

	IsHardware bool `json:"hardware_device" yaml:"hardware_device"`
	IsStream   bool `json:"stream_device" yaml:"stream_device"`

	// Mutable fields (can be updated via HTTP)

	Status  DeviceStatus `json:"status" yaml:"status"`
	IsMuted *bool        `json:"muted" yaml:"muted"`

	ToAddress   *AddressList `json:"to_address,omitempty" yaml:"to_address,omitempty"`
	FromAddress *AddressList `json:"from_address,omitempty" yaml:"from_address,omitempty"`

	// Internal fields (not visible via HTTP)

	NodeID   string `json:"-" yaml:"node_id,omitempty"`
	ModuleID string `json:"-" yaml:"module_id,omitempty"`
} // @name Device

func (d *Device) Dump() string {
	return dump(d)
}

func (d *Device) SetDefaults() {
	if d.Status == "" {
		d.Status = StatusEnabled
	}

	if d.IsMuted == nil {
		d.SetMuted(false)
	}

	if d.IsStream {
		switch d.Type {
		case Sink:
			if d.ToAddress == nil {
				d.ToAddress = &AddressList{}
			}
		case Source:
			if d.FromAddress == nil {
				d.FromAddress = &AddressList{}
			}
		}
	}
}

func (d *Device) SetMuted(val bool) {
	d.IsMuted = &val
}

func (d *Device) GetMuted() bool {
	return d.IsMuted != nil && *d.IsMuted
}

func (d *Device) GetEnabled() bool {
	return d.Status != StatusDisabled
}

func (d *Device) Compare(do *Device) int {
	switch {
	case d.Type < do.Type:
		return -1
	case d.Type > do.Type:
		return 1
	default:
		return deriveCompareDevice(d, do)
	}
}

func (d *Device) Equal(do *Device) bool {
	return deriveEqualDevice(d, do)
}

func (d *Device) Hash() uint64 {
	return deriveHashDevice(d)
}

func (d *Device) Clone() *Device {
	return deriveCloneDevice(d)
}
