// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package server

import (
	"io/fs"
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/manucorporat/sse"
	"github.com/swaggo/echo-swagger"

	"github.com/roc-streaming/rocd/admin"
	"github.com/roc-streaming/rocd/daemon/devices"
	"github.com/roc-streaming/rocd/daemon/events"
	"github.com/roc-streaming/rocd/daemon/models"
	"github.com/roc-streaming/rocd/daemon/streams"
	_ "github.com/roc-streaming/rocd/docs"
)

type Server struct {
	eh              *echo.Echo
	deviceManager   *devices.DeviceManager
	streamManager   *streams.StreamManager
	eventDispatcher *events.EventDispatcher
}

type Config struct {
	DeviceManager   *devices.DeviceManager
	StreamManager   *streams.StreamManager
	EventDispatcher *events.EventDispatcher
}

func New(config Config) *Server {
	srv := &Server{
		eh:              echo.New(),
		deviceManager:   config.DeviceManager,
		streamManager:   config.StreamManager,
		eventDispatcher: config.EventDispatcher,
	}

	srv.eh.HideBanner = true
	srv.eh.HidePort = true

	srv.eh.Pre(middleware.RemoveTrailingSlash())

	srv.eh.GET("/devices", srv.listDevices)
	srv.eh.GET("/devices/:device_uid", srv.getDevice)
	srv.eh.PUT("/devices/:device_uid", srv.updateDevice)

	srv.eh.GET("/streams", srv.listStreams)
	srv.eh.POST("/streams", srv.createStream)
	srv.eh.GET("/streams/:stream_uid", srv.getStream)
	srv.eh.PUT("/streams/:stream_uid", srv.updateStream)
	srv.eh.DELETE("/streams/:stream_uid", srv.deleteStream)

	srv.eh.GET("/stream_devices", srv.listStreamDevices)
	srv.eh.POST("/stream_devices", srv.createStreamDevice)
	srv.eh.GET("/stream_devices/:device_uid", srv.getStreamDevice)
	srv.eh.PUT("/stream_devices/:device_uid", srv.updateStreamDevice)
	srv.eh.DELETE("/stream_devices/:device_uid", srv.deleteStreamDevice)

	srv.eh.GET("/events", srv.listenEvents)

	adminFiles := admin.FileSystem()

	srv.eh.GET("/admin", echo.WrapHandler(
		http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			b, _ := fs.ReadFile(adminFiles, "index.html")
			w.Write(b)
		})))

	srv.eh.GET("/admin/*", echo.WrapHandler(
		http.StripPrefix("/admin", http.FileServerFS(adminFiles))))

	srv.eh.GET("/swagger/*", echoSwagger.WrapHandler)

	srv.eh.Use(requestLogger(
		"enter: ${remote_ip} ${method} ${uri}",
		"leave: ${remote_ip} ${method} ${uri} ${status} ${latency_human}",
	))

	return srv
}

func (s *Server) Serve(address string) error {
	log.Noticef("listening at http://%s", address)

	return s.eh.Start(address)
}

// @Summary	Show all devices
// @Success	200	{array}	models.Device
// @Router		/devices [get]
// @ID			listDevices
// @Tags		Devices
func (s *Server) listDevices(c echo.Context) error {
	devices, err := s.deviceManager.ListDevices()
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, devices)
}

// @Summary	Show device
// @Success	200			{object}	models.Device
// @Param		device_uid	path		string	true	"Device UID"
// @Router		/devices/{device_uid} [get]
// @ID			getDevice
// @Tags		Devices
func (s *Server) getDevice(c echo.Context) error {
	deviceUID := c.Param("device_uid")
	device, err := s.deviceManager.GetDevice(deviceUID)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, device)
}

// @Summary	Update device
// @Success	200			{object}	models.Device
// @Param		device_uid	path		string			true	"Device UID"
// @Param		device		body		models.Device	true	"Device JSON"
// @Router		/devices/{device_uid} [put]
// @ID			updateDevice
// @Tags		Devices
func (s *Server) updateDevice(c echo.Context) error {
	deviceUID := c.Param("device_uid")
	device := new(models.Device)
	if err := c.Bind(&device); err != nil {
		return err
	}
	device, err := s.deviceManager.UpdateDevice(deviceUID, device)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, device)
}

// @Summary	Show all streams
// @Success	200	{array}	models.Stream
// @Router		/streams [get]
// @ID			listStreams
// @Tags		Streams
func (s *Server) listStreams(c echo.Context) error {
	streams, err := s.streamManager.ListStreams()
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, streams)
}

// @Summary	Create stream
// @Success	200		{object}	models.Stream
// @Param		stream	body		models.Stream	true	"Stream JSON"
// @Router		/streams [post]
// @ID			createStream
// @Tags		Streams
func (s *Server) createStream(c echo.Context) error {
	stream := new(models.Stream)
	if err := c.Bind(&stream); err != nil {
		return err
	}
	stream, err := s.streamManager.CreateStream(stream)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, stream)
}

// @Summary	Show stream
// @Success	200			{object}	models.Stream
// @Param		stream_uid	path		string	true	"Stream UID"
// @Router		/streams/{stream_uid} [get]
// @ID			getStream
// @Tags		Streams
func (s *Server) getStream(c echo.Context) error {
	streamUID := c.Param("stream_uid")
	stream, err := s.streamManager.GetStream(streamUID)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, stream)
}

// @Summary	Update stream
// @Success	200			{object}	models.Stream
// @Param		stream_uid	path		string			true	"Stream UID"
// @Param		stream		body		models.Stream	true	"Stream JSON"
// @Router		/streams/{stream_uid} [put]
// @ID			updateStream
// @Tags		Streams
func (s *Server) updateStream(c echo.Context) error {
	streamUID := c.Param("stream_uid")
	stream := new(models.Stream)
	if err := c.Bind(&stream); err != nil {
		return err
	}
	stream, err := s.streamManager.UpdateStream(streamUID, stream)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, stream)
}

// @Summary	Delete stream
// @Success	204
// @Param		stream_uid	path	string	true	"Stream UID"
// @Router		/streams/{stream_uid} [delete]
// @ID			deleteStream
// @Tags		Streams
func (s *Server) deleteStream(c echo.Context) error {
	streamUID := c.Param("stream_uid")
	err := s.streamManager.DeleteStream(streamUID)
	if err != nil {
		return err
	}
	return c.NoContent(http.StatusNoContent)
}

// @Summary	Show all stream devices
// @Success	200	{array}	models.Device
// @Router		/stream_devices [get]
// @ID			listStreamDevices
// @Tags		Stream devices
func (s *Server) listStreamDevices(c echo.Context) error {
	devices, err := s.deviceManager.ListStreamDevices()
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, devices)
}

// @Summary	Create stream device
// @Success	200		{object}	models.Device
// @Param		device	body		models.Device	true	"Device JSON"
// @Router		/stream_devices [post]
// @ID			createStreamDevice
// @Tags		Stream devices
func (s *Server) createStreamDevice(c echo.Context) error {
	device := new(models.Device)
	if err := c.Bind(&device); err != nil {
		return err
	}
	device, err := s.deviceManager.CreateStreamDevice(device)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, device)
}

// @Summary	Show stream device
// @Success	200			{object}	models.Device
// @Param		device_uid	path		string	true	"Device UID"
// @Router		/stream_devices/{device_uid} [get]
// @ID			getStreamDevice
// @Tags		Stream devices
func (s *Server) getStreamDevice(c echo.Context) error {
	deviceUID := c.Param("device_uid")
	device, err := s.deviceManager.GetStreamDevice(deviceUID)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, device)
}

// @Summary	Update stream device
// @Success	200			{object}	models.Device
// @Param		device_uid	path		string			true	"Device UID"
// @Param		device		body		models.Device	true	"Device JSON"
// @Router		/stream_devices/{device_uid} [put]
// @ID			updateStreamDevice
// @Tags		Stream devices
func (s *Server) updateStreamDevice(c echo.Context) error {
	deviceUID := c.Param("device_uid")
	device := new(models.Device)
	if err := c.Bind(&device); err != nil {
		return err
	}
	device, err := s.deviceManager.UpdateStreamDevice(deviceUID, device)
	if err != nil {
		return err
	}
	return c.JSON(http.StatusOK, device)
}

// @Summary	Delete stream device
// @Success	204
// @Param		device_uid	path	string	true	"Device UID"
// @Router		/stream_devices/{device_uid} [delete]
// @ID			deleteStreamDevice
// @Tags		Stream devices
func (s *Server) deleteStreamDevice(c echo.Context) error {
	deviceUID := c.Param("device_uid")
	err := s.deviceManager.DeleteStreamDevice(deviceUID)
	if err != nil {
		return err
	}
	return c.NoContent(http.StatusNoContent)
}

// @Summary	Listen events
// @Success	200	{object}	models.Event
// @Produce	text/event-stream
// @Router		/events [get]
// @ID			listenEvents
// @Tags		Monitoring
func (s *Server) listenEvents(c echo.Context) error {
	ctx := c.Request().Context()

	w := c.Response()
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.WriteHeader(http.StatusOK)

	ln := s.eventDispatcher.Listen()
	defer ln.Stop()

	for {
		select {
		case event := <-ln.Chan():
			err := sse.Encode(w, sse.Event{
				Data: event,
			})
			if err != nil {
				return err
			}
			c.Response().Flush()

		case <-ctx.Done():
			return ctx.Err()
		}
	}
}
