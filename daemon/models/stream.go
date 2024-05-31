// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

type StreamType string // @name StreamType

const (
	SendStream StreamType = "send"
	RecvStream StreamType = "recv"
)

type Stream struct {
	// Immutable fields (assigned on creation)

	UID  string     `json:"stream_uid" yaml:"uid"`
	Type StreamType `json:"type" yaml:"type"`

	// Mutable fields (can be updated via HTTP)

	FromDevice string      `json:"from_device,omitempty" yaml:"from_device"`
	ToAddress  AddressList `json:"to_address,omitempty" yaml:"to_address"`

	ToDevice    string      `json:"to_device,omitempty" yaml:"to_device"`
	FromAddress AddressList `json:"from_address,omitempty" yaml:"from_address"`
} // @name Stream

func (s *Stream) Dump() string {
	return dump(s)
}

func (s *Stream) Compare(so *Stream) int {
	switch {
	case s.Type < so.Type:
		return -1
	case s.Type > so.Type:
		return 1
	default:
		return deriveCompareStream(s, so)
	}
}

func (s *Stream) Equal(so *Stream) bool {
	return deriveEqualStream(s, so)
}

func (s *Stream) Hash() uint64 {
	return deriveHashStream(s)
}

func (s *Stream) Clone() *Stream {
	return deriveCloneStream(s)
}
