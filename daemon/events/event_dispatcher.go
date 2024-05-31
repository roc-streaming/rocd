// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package events

import (
	"context"
	"sync"
	"sync/atomic"

	"golang.org/x/time/rate"

	"github.com/roc-streaming/rocd/daemon/models"
)

type state struct {
	devices models.DeviceMap
	streams models.StreamMap
}

func newState() *state {
	return &state{
		devices: make(models.DeviceMap),
		streams: make(models.StreamMap),
	}
}

type EventDispatcher struct {
	mu   sync.RWMutex
	cond *sync.Cond

	// State is immutable.
	// Whenever the state is changed, a new struct is allocated and
	// the pointer is updated.
	// It allows two things:
	//  - after obtaining pointer to state, you can work with it
	//    without a lock
	//  - to check whether the state changed, it's enough to check
	//    whether the pointer changed
	state *state
}

func NewEventDispatcher() (*EventDispatcher, error) {
	ed := &EventDispatcher{
		state: newState(),
	}

	ed.cond = sync.NewCond(ed.mu.RLocker())

	return ed, nil
}

func (ed *EventDispatcher) UpdateDevices(devices models.DeviceMap) {
	ed.mu.Lock()
	defer ed.mu.Unlock()

	if ed.state.devices.Equal(&devices) {
		return
	}

	log.Debugf("updating device list")

	newState := *ed.state
	newState.devices = make(models.DeviceMap)

	for _, dev := range devices {
		newState.devices[dev.UID] = dev
	}

	ed.state = &newState
	ed.cond.Broadcast()
}

func (ed *EventDispatcher) UpdateStreams(streams models.StreamMap) {
	ed.mu.Lock()
	defer ed.mu.Unlock()

	if ed.state.streams.Equal(&streams) {
		return
	}

	log.Debugf("updating stream list")

	newState := *ed.state
	newState.streams = make(models.StreamMap)

	for _, srm := range streams {
		newState.streams[srm.UID] = srm
	}

	ed.state = &newState
	ed.cond.Broadcast()
}

func (ed *EventDispatcher) Listen() *EventListener {
	el := &EventListener{
		edisp:   ed,
		eventCh: make(chan *models.Event, 500),
		closeCh: make(chan struct{}),
	}
	go el.run()
	return el
}

type EventListener struct {
	edisp *EventDispatcher

	// run() writes to this channel.
	// When listener is stopped, run() closes channel and exits.
	eventCh chan *models.Event

	// Stop() closes this channel.
	// When channel is closed, run() exits.
	closeCh chan struct{}

	// Ensure that we close channel only once.
	closed atomic.Bool
}

func (el *EventListener) Chan() <-chan *models.Event {
	return el.eventCh
}

func (el *EventListener) Stop() {
	if !el.closed.CompareAndSwap(false, true) {
		return
	}

	log.Debugf("stopping event listener")

	el.edisp.mu.Lock()
	defer el.edisp.mu.Unlock()

	close(el.closeCh)
	el.edisp.cond.Broadcast()
}

func (el *EventListener) run() {
	log.Debugf("opened event listener %p", el)
	defer log.Debugf("closed event listener %p", el)

	defer close(el.eventCh)

	lim := rate.NewLimiter(rate.Limit(eventInterval), 1)

	lastState := newState()

	for {
		// Rate-limit state updates.
		lim.Wait(context.Background())

		// Sleep until state struct changes to a new one.
		// State is immutable, so we can only check and copy a pointer.
		el.edisp.mu.RLock()
		for lastState == el.edisp.state {
			// Check if Stop() was called before going to sleep.
			if el.stopped() {
				el.edisp.mu.RUnlock()
				return
			}
			el.edisp.cond.Wait()
		}
		currState := el.edisp.state
		el.edisp.mu.RUnlock()

		// Report what new happened.
		el.report(lastState, currState)

		lastState = currState
	}
}

func (el *EventListener) report(lastState, currState *state) bool {
	nEvents := 0

	for _, dev := range lastState.devices {
		if _, ok := currState.devices[dev.UID]; !ok {
			el.write(models.Event{
				Type:      models.EventDeviceRemoved,
				DeviceUID: dev.UID,
			})
			nEvents++
		}
	}

	for _, dev := range currState.devices {
		if _, ok := lastState.devices[dev.UID]; !ok {
			el.write(models.Event{
				Type:      models.EventDeviceAdded,
				DeviceUID: dev.UID,
			})
			nEvents++
		}
	}

	for _, currDev := range currState.devices {
		if lastDev, _ := lastState.devices[currDev.UID]; lastDev != nil &&
			!lastDev.Equal(currDev) {
			el.write(models.Event{
				Type:      models.EventDeviceUpdated,
				DeviceUID: currDev.UID,
			})
			nEvents++
		}
	}

	if nEvents != 0 {
		log.Debugf("sent %d event(s) to listener %p", nEvents, el)
	}

	return true
}

func (el *EventListener) write(event models.Event) {
	select {
	case el.eventCh <- &event:
	case <-el.closeCh:
		// Don't block on eventCh if Stop() was called.
		// Probably nobody reads from eventCh already and we
		// can hang indefenitely.
	}
}

func (el *EventListener) stopped() bool {
	select {
	case <-el.closeCh:
		return true

	default:
		return false
	}
}
