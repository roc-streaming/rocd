// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

//go:build linux

package devices

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"os/exec"
	"regexp"
	"strings"
	"sync"
	"time"

	"github.com/Jeffail/gabs/v2"

	"github.com/roc-streaming/rocd/daemon/models"
	"github.com/roc-streaming/rocd/daemon/store"
)

type PipewireBackend struct {
	store *store.PersistStore

	streamDevMu  sync.Mutex
	streamDevMap map[string]*pwStreamDevice
	streamDevCh  chan *pwStreamDevice

	eventCh  chan backendEvent
	wakeupCh chan struct{}
}

func init() {
	backends = append(backends, newPipewireBackend())
}

func newPipewireBackend() *PipewireBackend {
	return &PipewireBackend{
		streamDevMap: make(map[string]*pwStreamDevice),
		streamDevCh:  make(chan *pwStreamDevice, 1024),
		eventCh:      make(chan backendEvent, 2),
		wakeupCh:     make(chan struct{}, 1),
	}
}

func (b *PipewireBackend) driver() models.DeviceDriver {
	return models.DriverPipewire
}

func (b *PipewireBackend) prio() backendPrio {
	return prioHigh
}

func (b *PipewireBackend) init(store *store.PersistStore) error {
	b.store = store

	cmd := exec.Command("pw-cli", "info", "all")
	err := cmd.Run()
	if err != nil {
		return errNotAvailable
	}

	go b.monitorEvents()

	return nil
}

func (b *PipewireBackend) listenDevices() <-chan backendEvent {
	return b.eventCh
}

func (b *PipewireBackend) fetchDevices() ([]*models.Device, error) {
	out, err := exec.Command("pw-dump").Output()
	if err != nil {
		return nil, err
	}

	dump, err := gabs.ParseJSON(out)
	if err != nil {
		return nil, err
	}

	allDevices := make([]*models.Device, 0)

	for _, devJson := range dump.Children() {
		dev := pwBuildDevice(devJson)
		if dev == nil {
			continue
		}

		if dev.IsStream {
			if savedDev, _ := b.store.LoadStreamDevice(dev.UID); savedDev != nil {
				// We can't retrieve some device parameters from PipeWire, so instead
				// we restore them from storage.
				switch dev.Type {
				case models.Sink:
					dev.ToAddress = savedDev.ToAddress
				case models.Source:
					dev.FromAddress = savedDev.FromAddress
				}
			} else {
				// If device is not managed by this rocd instance, don't show it as
				// stream device, because we don't know its settings and don't own
				// it (e.g. it may be created in pipewire config by hand).
				dev.IsStream = false
			}
		}

		allDevices = append(allDevices, dev)
	}

	return allDevices, nil
}

func (b *PipewireBackend) resetStreamDevice(dev *models.Device) error {
	if !dev.GetEnabled() {
		return nil
	}

	b.streamDevMu.Lock()

	_ = b.removeDevice(dev)
	err := b.createDevice(dev)

	b.streamDevMu.Unlock()

	if err != nil {
		return err
	}

	return b.waitDevice(dev)
}

func (b *PipewireBackend) destroyStreamDevice(dev *models.Device) error {
	b.streamDevMu.Lock()
	defer b.streamDevMu.Unlock()

	return b.removeDevice(dev)
}

func (b *PipewireBackend) createDevice(dev *models.Device) error {
	sd, err := b.startStreamDevice(dev)
	if err != nil {
		return fmt.Errorf("failed to start stream device: %w", err)
	}

	log.Debugf("adding stream device %q to map", dev.UID)
	b.streamDevMap[dev.UID] = sd

	return nil
}

func (b *PipewireBackend) removeDevice(dev *models.Device) error {
	if dev.NodeID != "" {
		cmd := command("pw-cli", "destroy", dev.NodeID)
		err := cmd.Run()
		if err != nil {
			log.Debugf("can't destroy pipewire node %q: %s", dev.NodeID, err.Error())
		}
	}

	sd, ok := b.streamDevMap[dev.UID]
	if ok {
		sd.stop()
		log.Debugf("removing stream device %q from map", dev.UID)
		delete(b.streamDevMap, dev.UID)
	} else {
		log.Debugf("device already removed from map")
	}

	dev.NodeID = ""

	return nil
}

func (b *PipewireBackend) waitDevice(dev *models.Device) error {
	timeoutCh := time.After(backendResponseTimeout)

	for {
		select {
		case <-b.wakeupCh:
			// Device added?
			devList, _ := b.fetchDevices()
			for _, newDev := range devList {
				if newDev.UID == dev.UID {
					return nil
				}
			}

			// Device failed?
			b.streamDevMu.Lock()
			sd, _ := b.streamDevMap[dev.UID]
			b.streamDevMu.Unlock()
			if sd == nil {
				return fmt.Errorf("failed to create device on backend")
			}

		case <-timeoutCh:
			_ = b.removeDevice(dev)
			return fmt.Errorf("device did not appear on backend during timeout")
		}
	}
}

func (b *PipewireBackend) applyEnabled(dev *models.Device) error {
	// PipeWire backend does not store enabled/disabled state of devices.
	// When device is disabled, it's just removed from backend.
	if dev.GetEnabled() {
		if err := b.resetStreamDevice(dev); err != nil {
			return err
		}
	} else {
		if err := b.destroyStreamDevice(dev); err != nil {
			return err
		}
		dev.NodeID = ""
	}
	return nil
}

func (b *PipewireBackend) applyMuted(dev *models.Device) error {
	if !dev.GetEnabled() {
		return nil
	}

	var subcmd string
	if dev.Type == models.Sink {
		subcmd = "set-sink-mute"
	} else {
		subcmd = "set-source-mute"
	}

	var value string
	if dev.GetMuted() {
		value = "1"
	} else {
		value = "0"
	}

	// We should use pw-cli instead of pactl to avoid dependency on pipewire-pulse,
	// but I wasn't able to make mute/unmute working via pw-cli.
	cmd := command("pactl", subcmd, dev.SystemName, value)
	err := cmd.Run()
	if err != nil {
		return fmt.Errorf("failed to change device mute state: %w", err)
	}

	return nil
}

func (b *PipewireBackend) applyAddress(dev *models.Device) error {
	if !dev.GetEnabled() {
		return nil
	}

	// We can't change device settings on fly, so we recreate
	// device with new settings instead.
	return b.resetStreamDevice(dev)
}

var (
	pwEventRe = regexp.MustCompile(`^([a-z]+):$`)
	pwNodeRe  = regexp.MustCompile(`^\s+id:\s*(\d+)$`)
	pwTypeRe  = regexp.MustCompile(`^\s+type:\s*([a-zA-Z:]+).*$`)
)

func (b *PipewireBackend) triggerDeviceEvent(ev backendEvent) {
	switch ev.eventType {
	case eventDeviceListUpdated:
		select {
		case b.eventCh <- ev:
		default:
			// If channel is full, drop event because device manager guaranteedely
			// will update device list update soon. This approach allows us to
			// squash multiple subsequent updates into once to reduce noise.
		}
	default:
		// Other events shouldn't be lost, so we block here.
		b.eventCh <- ev
	}

	select {
	case b.wakeupCh <- struct{}{}:
	default:
	}
}

func (b *PipewireBackend) triggerStreamEvent() {
	select {
	case b.wakeupCh <- struct{}{}:
	default:
	}
}

func (b *PipewireBackend) monitorEvents() {
	firstConnect := true

	for {
		func() {
			cmd := exec.Command("pw-mon")
			out, err := cmd.StdoutPipe()
			if err != nil {
				panic(err)
			}
			defer out.Close()

			if err = cmd.Start(); err != nil {
				if firstConnect {
					firstConnect = false
					log.Debugf("can't establish connection to pipewire: %s", err)
				}
				time.Sleep(backendReconnectInterval)
				return
			}

			b.scanEvents(out)
		}()
	}
}

func (b *PipewireBackend) scanEvents(out io.Reader) {
	nodeTypes := make(map[string]string)

	lastEvent := ""
	lastNode := ""
	lastType := ""

	scanner := bufio.NewScanner(out)
	nLines := uint64(0)

	for scanner.Scan() {
		line := scanner.Text()
		nLines++

		if nLines == 1 {
			log.Debugf("established connection to pipewire")
			b.triggerDeviceEvent(backendEvent{
				eventType: eventDeviceListWiped,
			})
		}

		// Event name.
		if m := pwEventRe.FindStringSubmatch(line); len(m) != 0 {
			lastEvent = m[1]
			continue
		}

		// Node id and type.
		if lastEvent != "" {
			if m := pwNodeRe.FindStringSubmatch(line); len(m) != 0 {
				lastNode = m[1]
			} else if m := pwTypeRe.FindStringSubmatch(line); len(m) != 0 {
				lastType = m[1]
			}

			// Node added or changed, remember mapping of id to type.
			// If it's device node, generate event.
			if (lastEvent == "added" || lastEvent == "changed") &&
				lastNode != "" && lastType != "" {
				if lastType == "PipeWire:Interface:Node" {
					log.Debugf("event: %s node %s %s", lastEvent, lastNode, lastType)
					b.triggerDeviceEvent(backendEvent{
						eventType: eventDeviceListUpdated,
					})
				}
				nodeTypes[lastNode] = lastType
				lastEvent = ""
				lastNode = ""
				lastType = ""
				continue
			}

			// Node removed, clear mapping.
			// If it's device node, generate event.
			if lastEvent == "removed" && lastNode != "" {
				if remType, _ := nodeTypes[lastNode]; remType == "PipeWire:Interface:Node" {
					log.Debugf("event: %s node %s %s", lastEvent, lastNode, remType)
					devUID := ""
					for _, dev := range b.store.LoadStreamDevices() {
						if dev.NodeID == lastNode {
							devUID = dev.UID
							break
						}
					}
					if devUID != "" {
						b.triggerDeviceEvent(backendEvent{
							eventType: eventDeviceRemoved,
							deviceUID: devUID,
						})
					} else {
						b.triggerDeviceEvent(backendEvent{
							eventType: eventDeviceListUpdated,
						})
					}
				}
				delete(nodeTypes, lastNode)
				lastEvent = ""
				lastNode = ""
				lastType = ""
				continue
			}
		}
	}

	if nLines > 0 {
		if scanner.Err() != nil {
			log.Debugf("lost connection to pipewire: %s", scanner.Err())
		} else {
			log.Debugf("lost connection to pipewire")
		}
	}
}

type pwStreamDevice struct {
	cmd *exec.Cmd
	out io.ReadCloser
	uid string
}

func (b *PipewireBackend) startStreamDevice(dev *models.Device) (*pwStreamDevice, error) {
	log.Debugf("starting stream device %q", dev.UID)

	module := ""
	if dev.Type == models.Sink {
		module = "libpipewire-module-roc-sink"
	} else {
		module = "libpipewire-module-roc-source"
	}

	args, err := pwModuleArgs(dev)
	if err != nil {
		return nil, err
	}

	sd := &pwStreamDevice{
		cmd: command("pw-cli", "-m", "load-module", module, args),
		uid: dev.UID,
	}

	sd.out, err = sd.cmd.StderrPipe()
	if err != nil {
		panic(err)
	}

	if err := sd.cmd.Start(); err != nil {
		sd.out.Close()
		return nil, err
	}

	go func() {
		defer sd.out.Close()

		sd.run()

		func() {
			b.streamDevMu.Lock()
			defer b.streamDevMu.Unlock()

			curDev, _ := b.streamDevMap[sd.uid]
			if curDev == sd {
				log.Debugf("asynchronously removing stream device %q from map", sd.uid)
				delete(b.streamDevMap, sd.uid)
			}
		}()

		b.triggerStreamEvent()
	}()

	return sd, err
}

func (sd *pwStreamDevice) run() {
	scanner := bufio.NewScanner(sd.out)

	for scanner.Scan() {
		line := scanner.Text()

		if strings.Contains(line, "Error:") {
			log.Debugf("stream device reported error")
			return
		}
	}

	if scanner.Err() != nil {
		log.Debugf("stream device exited: %s", scanner.Err())
		return
	}

	log.Debugf("stream device exited without error")
}

func (sd *pwStreamDevice) stop() {
	if sd.cmd == nil || sd.cmd.Process == nil {
		return
	}
	log.Debugf("stopping stream device %q", sd.uid)
	sd.cmd.Process.Kill()
}

func pwModuleArgs(dev *models.Device) (string, error) {
	var addr *models.Address
	var kind string

	switch dev.Type {
	case models.Sink:
		if dev.ToAddress.Len() != 1 {
			return "", errors.New(
				"pipewire send stream device requires exactly one element in 'to_address'")
		}
		addr = dev.ToAddress.At(0)
		kind = "remote"

	case models.Source:
		if dev.FromAddress.Len() != 1 {
			return "", errors.New(
				"pipewire receive stream device requires exactly one element in 'from_address'")
		}
		addr = dev.FromAddress.At(0)
		kind = "local"
	}

	if addr.AudioSource == "" {
		return "", errors.New(
			"pipewire stream device requires 'audio_source' to be present")
	}

	if (addr.AudioRepair != "" && addr.AudioRepair.IP() != addr.AudioSource.IP()) ||
		(addr.AudioControl != "" && addr.AudioControl.IP() != addr.AudioSource.IP()) {
		return "", errors.New(
			"pipewire stream device requires 'audio_source', 'audio_repair'," +
				" and 'audio_control' URIs to use the same hostname")
	}

	fecCode := addr.AudioSource.Fec()
	if fecCode == "" {
		fecCode = "disable"
	}

	var sb strings.Builder

	fmt.Fprintf(&sb, "{\n")

	fmt.Fprintf(&sb, "  sink.name = %q\n", dev.DisplayName)
	fmt.Fprintf(&sb, "  sink.props = {\n")
	fmt.Fprintf(&sb, "    node.name = %q\n", dev.SystemName)
	fmt.Fprintf(&sb, "    node.description = %q\n", dev.DisplayName)
	fmt.Fprintf(&sb, "  }\n")

	fmt.Fprintf(&sb, "  %s.ip = %s\n", kind, addr.AudioSource.IP())
	fmt.Fprintf(&sb, "  %s.source.port = %s\n", kind, addr.AudioSource.Port())
	if addr.AudioRepair != "" {
		fmt.Fprintf(&sb, "  %s.repair.port = %s\n", kind, addr.AudioRepair.Port())
	}
	if addr.AudioControl != "" {
		fmt.Fprintf(&sb, "  %s.control.port = %s\n", kind, addr.AudioControl.Port())
	}

	fmt.Fprintf(&sb, "  fec.code = %s\n", fecCode)

	sb.WriteString("}")

	return sb.String(), nil
}

func pwBuildDevice(devJson *gabs.Container) *models.Device {
	var dev models.Device

	typ, _ := devJson.Path("type").Data().(string)
	if typ != "PipeWire:Interface:Node" {
		return nil
	}

	if id := devJson.Path("id").String(); id != "" {
		dev.NodeID = id
	} else {
		return nil
	}

	if name, ok := devJson.Path("info.props.node~1name").Data().(string); ok {
		dev.SystemName = name
	} else {
		return nil
	}

	if desc, ok := devJson.Path("info.props.node~1description").Data().(string); ok {
		dev.DisplayName = desc
	} else {
		return nil
	}

	mediaClass, _ := devJson.Path("info.props.media~1class").Data().(string)
	switch mediaClass {
	case "Audio/Sink":
		dev.Type = models.Sink
	case "Audio/Source":
		dev.Type = models.Source
	default:
		return nil
	}

	dev.UID = makeDeviceUID(dev.SystemName)
	dev.Driver = models.DriverPipewire

	isVirtual, _ := devJson.Path("info.props.node~1virtual").Data().(bool)
	dev.IsHardware = !isVirtual

	mediaName, _ := devJson.Path("info.props.media~1name").Data().(string)
	if strings.Contains(mediaName, "roc-sink") || strings.Contains(mediaName, "roc-source") {
		dev.IsStream = true
	}

	for _, propJson := range devJson.Path("info.params.Props").Children() {
		if mute, ok := propJson.Path("mute").Data().(bool); ok {
			dev.SetMuted(mute)
		}
	}

	return &dev
}
