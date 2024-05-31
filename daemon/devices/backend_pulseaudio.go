// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

//go:build linux

package devices

import (
	"bufio"
	"bytes"
	"errors"
	"fmt"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
	"sync/atomic"
	"time"

	"github.com/Jeffail/gabs/v2"

	"github.com/roc-streaming/rocd/daemon/models"
	"github.com/roc-streaming/rocd/daemon/store"
)

type PulseaudioBackend struct {
	store   *store.PersistStore
	eventCh chan backendEvent
}

func init() {
	backends = append(backends, newPulseaudioBackend())
}

func newPulseaudioBackend() *PulseaudioBackend {
	return &PulseaudioBackend{
		eventCh: make(chan backendEvent, 2),
	}
}

func (b *PulseaudioBackend) driver() models.DeviceDriver {
	return models.DriverPulseaudio
}

func (b *PulseaudioBackend) prio() backendPrio {
	return prioMedium
}

func (b *PulseaudioBackend) init(store *store.PersistStore) error {
	b.store = store

	cmd := exec.Command("pactl", "info")
	err := cmd.Run()
	if err != nil {
		return errNotAvailable
	}

	go b.monitorEvents()

	return nil
}

func (b *PulseaudioBackend) listenDevices() <-chan backendEvent {
	return b.eventCh
}

func (b *PulseaudioBackend) fetchDevices() ([]*models.Device, error) {
	allDevices := make([]*models.Device, 0)

	for _, dir := range []deviceDir{outputDir, inputDir} {
		ds, err := pulseBuildDeviceList(dir)
		if err != nil {
			return nil, err
		}

		for _, dev := range ds {
			if dev == nil {
				continue
			}

			if dev.IsStream {
				if savedDev, _ := b.store.LoadStreamDevice(dev.UID); savedDev != nil {
					// We can't retrieve some device parameters from PulseAudio, so instead
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
	}

	return allDevices, nil
}

func (b *PulseaudioBackend) resetStreamDevice(dev *models.Device) error {
	if !dev.GetEnabled() {
		return nil
	}

	if dev.ModuleID != "" {
		_ = b.destroyStreamDevice(dev)
	}

	module := ""
	if dev.Type == models.Sink {
		module = "module-roc-sink"
	} else {
		module = "module-roc-source"
	}

	args, err := pulseModuleArgs(dev)
	if err != nil {
		return err
	}

	cmdline := append([]string{"load-module", module}, args...)

	cmd := command("pactl", cmdline...)
	err = cmd.Run()
	if err != nil {
		return fmt.Errorf("failed to load device module: %w", err)
	}

	return nil
}

func (b *PulseaudioBackend) destroyStreamDevice(dev *models.Device) error {
	if dev.ModuleID == "" {
		return errors.New("failed to delete device: owner module not known")
	}

	cmd := command("pactl", "unload-module", dev.ModuleID)
	err := cmd.Run()
	if err != nil {
		return fmt.Errorf("failed to unload device module: %w", err)
	}

	dev.ModuleID = ""

	return nil
}

func (b *PulseaudioBackend) applyEnabled(dev *models.Device) error {
	// PulseAudio backend does not store enabled/disabled state of devices.
	// When device is disabled, it's just removed from backend.
	if dev.GetEnabled() {
		if err := b.resetStreamDevice(dev); err != nil {
			return err
		}
	} else {
		if err := b.destroyStreamDevice(dev); err != nil {
			return err
		}
	}
	return nil
}

func (b *PulseaudioBackend) applyMuted(dev *models.Device) error {
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

	cmd := command("pactl", subcmd, dev.SystemName, value)
	err := cmd.Run()
	if err != nil {
		return fmt.Errorf("failed to change device mute state: %w", err)
	}

	return nil
}

func (b *PulseaudioBackend) applyAddress(dev *models.Device) error {
	if !dev.GetEnabled() {
		return nil
	}

	// We can't change device settings on fly, so we recreate
	// device with new settings instead.
	return b.resetStreamDevice(dev)
}

func (b *PulseaudioBackend) monitorEvents() {
	firstConnect := true

	for {
		func() {
			cmd := exec.Command("pactl", "subscribe")
			out, err := cmd.StdoutPipe()
			if err != nil {
				panic(err)
			}
			defer out.Close()

			if err = cmd.Start(); err != nil {
				if firstConnect {
					firstConnect = false
					log.Debugf("can't establish connection to pulseaudio: %s", err)
				}
				time.Sleep(backendReconnectInterval)
				return
			}

			var runPing atomic.Bool

			runPing.Store(true)
			defer runPing.Store(false)

			// Ping pulseaudio until we receive first event.
			// We need it to generate eventDeviceListWiped when we know that
			// we've successfully connected to PulseAudio.
			go func() {
				for runPing.Load() {
					time.Sleep(backendPingInterval)
					exec.Command("pactl", "info").Run()
				}
			}()

			scanner := bufio.NewScanner(out)
			nEvents := 0

			for scanner.Scan() {
				line := scanner.Text()
				line = strings.ToLower(line)

				if strings.Contains(line, "connection failure") ||
					strings.Contains(line, "connection refused") {
					break
				}

				event, isEvent := b.parseEvent(line)
				if !isEvent {
					continue
				}

				nEvents++
				if nEvents == 1 {
					runPing.Store(false) // Stop ping.
					log.Debugf("established connection to pulseaudio")
					b.triggerEvent(backendEvent{
						eventType: eventDeviceListWiped,
					})
				}

				if event == nil {
					continue
				}

				b.triggerEvent(*event)
			}

			if nEvents > 0 {
				if scanner.Err() != nil {
					log.Debugf("lost connection to pulseaudio: %s", scanner.Err())
				} else {
					log.Debugf("lost connection to pulseaudio")
				}
			}
		}()
	}
}

var (
	pulseEventRe = regexp.MustCompile(`^event\s+'(\w+)'\s+on\s+([a-z-]+)\s+#(\d+)$`)
)

func (b *PulseaudioBackend) parseEvent(text string) (*backendEvent, bool) {
	m := pulseEventRe.FindStringSubmatch(text)
	if len(m) == 0 {
		return nil, false
	}

	evType, objType, nodeID := m[1], m[2], m[3]
	if objType != "source" && objType != "sink" {
		return nil, true
	}

	log.Debugf("event: %s %s %s", evType, objType, nodeID)

	if evType == "remove" {
		for _, dev := range b.store.LoadStreamDevices() {
			if dev.NodeID == nodeID {
				ev := &backendEvent{
					eventType: eventDeviceRemoved,
					deviceUID: dev.UID,
				}
				return ev, true
			}
		}
	}

	ev := &backendEvent{
		eventType: eventDeviceListUpdated,
	}
	return ev, true
}

func (b *PulseaudioBackend) triggerEvent(ev backendEvent) {
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
}

func pulseModuleArgs(dev *models.Device) ([]string, error) {
	var addr *models.Address
	var kind string

	switch dev.Type {
	case models.Sink:
		if dev.ToAddress.Len() != 1 {
			return nil, errors.New(
				"pulseaudio send stream device requires exactly one element in 'to_address'")
		}
		addr = dev.ToAddress.At(0)
		kind = "remote"

	case models.Source:
		if dev.FromAddress.Len() != 1 {
			return nil, errors.New(
				"pulseaudio receive stream device requires exactly one element in 'from_address'")
		}
		addr = dev.FromAddress.At(0)
		kind = "local"
	}

	if addr.AudioSource == "" {
		return nil, errors.New(
			"pulseaudio stream device requires 'audio_source' to be present")
	}

	if (addr.AudioRepair != "" && addr.AudioRepair.IP() != addr.AudioSource.IP()) ||
		(addr.AudioControl != "" && addr.AudioControl.IP() != addr.AudioSource.IP()) {
		return nil, errors.New(
			"pulseaudio stream device requires 'audio_source', 'audio_repair'," +
				" and 'audio_control' URIs to use the same hostname")
	}

	fecCode := addr.AudioSource.Fec()
	if fecCode == "" {
		fecCode = "disable"
	}

	args := []string{}

	if dev.Type == models.Sink {
		args = append(args, fmt.Sprintf("sink_name=%s", dev.SystemName))
		args = append(args, fmt.Sprintf("sink_properties='device.description=%q'",
			dev.DisplayName))
	} else {
		args = append(args, fmt.Sprintf("source_name=%s", dev.SystemName))
		args = append(args, fmt.Sprintf("source_properties='device.description=%q'",
			dev.DisplayName))
	}

	args = append(args, fmt.Sprintf("%s_ip=%s", kind, addr.AudioSource.IP()))
	args = append(args, fmt.Sprintf("%s_source_port=%s", kind, addr.AudioSource.Port()))
	if addr.AudioRepair != "" {
		args = append(args, fmt.Sprintf("%s_repair_port=%s", kind, addr.AudioRepair.Port()))
	}
	if addr.AudioControl != "" {
		args = append(args, fmt.Sprintf("%s_control_port=%s", kind, addr.AudioControl.Port()))
	}

	args = append(args, fmt.Sprintf("fec_encoding=%s", fecCode))

	return args, nil
}

func pulseBuildDeviceList(dir deviceDir) ([]*models.Device, error) {
	var subcmd string
	if dir == outputDir {
		subcmd = "sinks"
	} else {
		subcmd = "sources"
	}

	var out []byte

	if pulseSupportsJSON() {
		var err error
		out, err = exec.Command("pactl", "-fjson", "list", subcmd).Output()
		if err != nil {
			return nil, err
		}
	} else {
		var err error
		out, err = exec.Command("pactl", "list", subcmd).Output()
		if err != nil {
			return nil, err
		}

		// Compatibility with older pulseaudio versions.
		// Less reliable parsing.
		out, err = pulseReparseToJSON(out)
		if err != nil {
			return nil, err
		}
	}

	dump, err := gabs.ParseJSON(out)
	if err != nil {
		return nil, err
	}

	devices := make([]*models.Device, 0)

	for _, devJson := range dump.Children() {
		dev := pulseBuildDevice(dir, devJson)
		if dev == nil {
			continue
		}
		devices = append(devices, dev)
	}

	return devices, nil
}

func pulseBuildDevice(dir deviceDir, devJson *gabs.Container) *models.Device {
	var dev models.Device

	if name, ok := devJson.Path("name").Data().(string); ok {
		dev.SystemName = name
	} else {
		return nil
	}

	if desc, ok := devJson.Path("description").Data().(string); ok {
		dev.DisplayName = desc
	} else {
		return nil
	}

	class, _ := devJson.Path("properties.device~1class").Data().(string)
	if class == "monitor" {
		return nil
	}

	if dir == outputDir {
		dev.Type = models.Sink
	} else {
		dev.Type = models.Source
	}

	dev.UID = makeDeviceUID(dev.SystemName)
	dev.Driver = models.DriverPulseaudio

	if index, ok := devJson.Path("index").Data().(string); ok {
		dev.NodeID = index
	}

	if module, ok := devJson.Path("owner_module").Data().(string); ok {
		dev.ModuleID = module
	}

	for _, flag := range devJson.Path("flags").Children() {
		if s, _ := flag.Data().(string); s == "HARDWARE" {
			dev.IsHardware = true
		}
	}

	driver, _ := devJson.Path("driver").Data().(string)
	switch driver {
	case "roc_sender", "roc_receiver", "roc-sink", "roc-source":
		dev.IsStream = true
	}

	if mute, ok := devJson.Path("mute").Data().(bool); ok {
		dev.SetMuted(mute)
	}

	return &dev
}

func pulseSupportsJSON() bool {
	out, err := exec.Command("pactl", "--help").CombinedOutput()
	if err != nil {
		return false
	}

	help := string(out)

	if !strings.Contains(help, "--format") || !strings.Contains(help, "json") {
		return false
	}

	return true
}

var (
	pulseBeginRe   = regexp.MustCompile(`^\w+\s+#(\d+)$`)
	pulseEndRe     = regexp.MustCompile(`^$`)
	pulseFieldRe   = regexp.MustCompile(`^\s+([A-Z][A-Za-z0-9 ]+):\s*(.*)$`)
	pulsePropRe    = regexp.MustCompile(`^\s+([A-Za-z0-9_.]+)\s*=\s*"(.*)"\s*$`)
	pulseFormatRe  = regexp.MustCompile(`^\s+(\S+)\s*$`)
	pulseVolumeRe  = regexp.MustCompile(`^\s*([0-9.]+)\s*/\s*([0-9.%]+)\s*/\s*(.+)$`)
	pulseLatencyRe = regexp.MustCompile(`^\s*([0-9.]+)\s*usec.*configured\s*([0-9.]+)\s*usec$`)
)

func pulseReparseToJSON(text []byte) ([]byte, error) {
	scanner := bufio.NewScanner(bytes.NewReader(text))

	var (
		devList *gabs.Container
		dev     *gabs.Container
	)

	devList = gabs.New()

	lastField := ""

	for scanner.Scan() {
		line := scanner.Text()

		// device begin
		if m := pulseBeginRe.FindStringSubmatch(line); len(m) != 0 {
			if dev != nil {
				devList.ArrayAppend(dev, ".")
				dev = nil
			}
			dev = gabs.New()
			dev.Set(m[1], "index")
			continue
		}

		// device field
		if m := pulseFieldRe.FindStringSubmatch(line); len(m) != 0 {
			key, value := m[1], m[2]
			key = strings.ToLower(key)
			key = strings.Replace(key, " ", "_", -1)

			lastField = key

			switch key {
			case "properties":
			case "formats":
			case "flags":
				dev.Set(strings.Fields(value), key)
			case "mute":
				dev.Set(strings.ToLower(value) == "on" ||
					strings.ToLower(value) == "yes", key)
			case "volume":
				for _, chSpec := range strings.Split(value, ",") {
					parts := strings.SplitN(chSpec, ":", 2)
					if len(parts) == 2 {
						chName := strings.TrimSpace(parts[0])
						if m := pulseVolumeRe.FindStringSubmatch(parts[1]); len(m) != 0 {
							num, _ := strconv.ParseFloat(m[1], 64)
							dev.Set(num, "volume", chName, "value")
							dev.Set(m[2], "volume", chName, "value_percent")
							dev.Set(m[3], "volume", chName, "db")
						}
					}
				}
			case "base_volume":
				if m := pulseVolumeRe.FindStringSubmatch(value); len(m) != 0 {
					num, _ := strconv.ParseFloat(m[1], 64)
					dev.Set(num, "base_volume", "value")
					dev.Set(m[2], "base_volume", "value_percent")
					dev.Set(m[3], "base_volume", "db")
				}
			case "latency":
				if m := pulseLatencyRe.FindStringSubmatch(value); len(m) != 0 {
					actual, _ := strconv.ParseFloat(m[1], 64)
					configured, _ := strconv.ParseFloat(m[2], 64)
					dev.Set(actual, "latency", "actual")
					dev.Set(configured, "latency", "configured")
				}
			default:
				dev.Set(value, key)
			}

			continue
		}

		// device property
		if dev != nil && lastField == "properties" {
			if m := pulsePropRe.FindStringSubmatch(line); len(m) != 0 {
				dev.Set(m[2], "properties", m[1])
				continue
			}
		}

		// device format
		if dev != nil && lastField == "formats" {
			if m := pulseFormatRe.FindStringSubmatch(line); len(m) != 0 {
				dev.ArrayAppend(m[1], "formats")
				continue
			}
		}

		// device end
		if pulseEndRe.MatchString(line) {
			if dev != nil {
				devList.ArrayAppend(dev, ".")
				dev = nil
			}
			continue
		}
	}

	if dev != nil {
		devList.ArrayAppend(dev, ".")
		dev = nil
	}

	result := devList.S(".").StringIndent("", "  ")

	return []byte(result), nil
}
