// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

import (
	"fmt"
)

type Address struct {
	AudioSource  URI `json:"audio_source" yaml:"audio_source"`
	AudioRepair  URI `json:"audio_repair" yaml:"audio_repair"`
	AudioControl URI `json:"audio_control" yaml:"audio_control"`
} // @name Address

func (a *Address) Validate() error {
	if a.AudioSource != "" {
		if err := a.AudioSource.Validate(IfaceAudioSource); err != nil {
			return err
		}
	}
	if a.AudioRepair != "" {
		if err := a.AudioRepair.Validate(IfaceAudioRepair); err != nil {
			return err
		}
	}
	if a.AudioControl != "" {
		if err := a.AudioControl.Validate(IfaceAudioControl); err != nil {
			return err
		}
	}

	if a.AudioSource == "" {
		return fmt.Errorf("'audio_source' uri is missing")
	}

	if a.AudioRepair == "" {
		if a.AudioSource.Fec() != "" {
			return fmt.Errorf("'audio_source' uri protocol implies fec scheme %q"+
				" for repair packets, but 'audio_repair' uri is missing", a.AudioSource.Fec())
		}
	} else {
		if a.AudioSource.Fec() != a.AudioRepair.Fec() {
			return fmt.Errorf("'audio_source' uri protocol implies fec scheme %q"+
				" for repair packets, but 'audio_repair' uri implies fec scheme %q",
				a.AudioSource.Fec(),
				a.AudioRepair.Fec())
		}
	}

	return nil
}

func (a *Address) Compare(ao *Address) int {
	return deriveCompareAddress(a, ao)
}

func (a *Address) Equal(ao *Address) bool {
	return deriveEqualAddress(a, ao)
}

func (a *Address) Hash() uint64 {
	return deriveHashAddress(a)
}

func (a *Address) Clone() *Address {
	return deriveCloneAddress(a)
}
