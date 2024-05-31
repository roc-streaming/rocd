// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

type AddressList []Address

func (l *AddressList) Validate() error {
	if l == nil {
		return nil
	}
	for _, a := range *l {
		if err := a.Validate(); err != nil {
			return err
		}
	}
	return nil
}

func (l *AddressList) Len() int {
	if l == nil {
		return 0
	}
	return len(*l)
}

func (l *AddressList) At(index int) *Address {
	if l == nil {
		panic("address list is nil")
	}
	if *l == nil {
		panic("address list is pointer to nil")
	}
	return &(*l)[index]
}

func (l *AddressList) Compare(lo *AddressList) int {
	return deriveCompareAddressList(l, lo)
}

func (l *AddressList) Equal(lo *AddressList) bool {
	return deriveEqualAddressList(l, lo)
}

func (l *AddressList) Hash() uint64 {
	return deriveHashAddressList(l)
}

func (l *AddressList) Clone() *AddressList {
	return deriveCloneAddressList(l)
}
