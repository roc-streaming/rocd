// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package logs

import (
	"bufio"
	"os"
	"sync"

	"github.com/fatih/color"
	"github.com/op/go-logging"
)

var levelNames = []string{
	"EE",
	"EE",
	"WW",
	"NN",
	"II",
	"DD",
}

var levelColors = []*color.Color{
	color.New(color.FgRed),
	color.New(color.FgRed),
	color.New(color.FgYellow),
	color.New(color.FgHiGreen),
	color.New(color.FgHiBlue),
	color.New(color.Reset),
}

var timeColor = color.New(color.FgWhite)

type backend struct {
	mu sync.Mutex
	wr *bufio.Writer
}

func newBackend() *backend {
	return &backend{
		wr: bufio.NewWriter(os.Stderr),
	}
}

func (b *backend) Log(level logging.Level, calldepth int, rec *logging.Record) error {
	b.mu.Lock()
	defer b.mu.Unlock()

	timeColor.Fprint(b.wr, rec.Time.Format("15:04:05.000"))
	b.wr.WriteString(" [")
	levelColors[level].Fprint(b.wr, levelNames[level])
	b.wr.WriteString("] ")
	b.wr.WriteString(rec.Module)
	b.wr.WriteString(": ")
	levelColors[level].Fprint(b.wr, rec.Message())
	b.wr.WriteString("\n")
	b.wr.Flush()

	return nil
}

func init() {
	logging.SetBackend(newBackend())
}
