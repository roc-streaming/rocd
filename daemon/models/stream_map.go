// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

type StreamMap map[string]*Stream

func (m *StreamMap) Compare(mo *StreamMap) int {
	return deriveCompareStreamMap(m, mo)
}

func (m *StreamMap) Equal(mo *StreamMap) bool {
	return deriveEqualStreamMap(m, mo)
}

func (m *StreamMap) Hash() uint64 {
	return deriveHashStreamMap(m)
}

func (m *StreamMap) Clone() *StreamMap {
	return deriveCloneStreamMap(m)
}
