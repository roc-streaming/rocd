// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package server

import (
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
)

type funcWriter func(s string)

func (fn funcWriter) Write(p []byte) (n int, err error) {
	fn(string(p))
	return len(p), nil
}

func requestLogger(beforeFmt string, afterFmt string) echo.MiddlewareFunc {
	beforeMw := middleware.LoggerWithConfig(middleware.LoggerConfig{
		Format:           beforeFmt,
		CustomTimeFormat: "15:04:05.000",
		Output: funcWriter(func(s string) {
			log.Info(s)
		}),
	})
	afterMw := middleware.LoggerWithConfig(middleware.LoggerConfig{
		Format:           afterFmt,
		CustomTimeFormat: "15:04:05.000",
		Output: funcWriter(func(s string) {
			log.Info(s)
		}),
	})

	return func(next echo.HandlerFunc) echo.HandlerFunc {
		beforeFn := beforeMw(func(c echo.Context) error {
			return nil
		})
		afterFn := afterMw(func(c echo.Context) error {
			return nil
		})
		return func(c echo.Context) error {
			beforeFn(c)

			err := next(c)
			if err != nil {
				c.Error(err)
			}

			afterFn(c)

			return err
		}
	}
}
