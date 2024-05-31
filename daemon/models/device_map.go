// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

type DeviceMap map[string]*Device

func (m *DeviceMap) Compare(mo *DeviceMap) int {
	return deriveCompareDeviceMap(m, mo)
}

func (m *DeviceMap) Equal(mo *DeviceMap) bool {
	return deriveEqualDeviceMap(m, mo)
}

func (m *DeviceMap) Hash() uint64 {
	return deriveHashDeviceMap(m)
}

func (m *DeviceMap) Clone() *DeviceMap {
	return deriveCloneDeviceMap(m)
}
