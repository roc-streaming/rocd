// Copyright (c) 2024 Roc Streaming authors
// Licensed under MPL-2.0

package models

import (
	"fmt"
	"net"
	"net/url"
)

type URI string

func (s URI) Validate(iface Interface) error {
	u, err := url.Parse(string(s))
	if err != nil {
		return fmt.Errorf("invalid uri %q: %w", s, err)
	}

	if u.Opaque != "" {
		return fmt.Errorf("invalid uri %q: unsupported format", s)
	}

	switch s.Scheme() {
	case "rtp", "rtp+rs8m", "rs8m", "rtp+ldpc", "ldpc", "rtcp":
		if s.User() != "" {
			return fmt.Errorf("invalid uri %q: 'user' component not supported", s)
		}
		if s.Host() == "" {
			return fmt.Errorf("invalid uri %q: 'host' component missing", s)
		}
		if s.Port() == "" {
			return fmt.Errorf("invalid uri %q: 'port' component missing", s)
		}
		if s.Resource() != "" {
			return fmt.Errorf("invalid uri %q: 'resource' component not supported", s)
		}

	case "rtsp":
		if s.User() != "" {
			return fmt.Errorf("invalid uri %q: 'user' component not supported", s)
		}

	default:
		return fmt.Errorf("invalid uri %q: unknown scheme %q", s, u.Scheme)
	}

	switch iface {
	case IfaceAudioSource:
		switch s.Scheme() {
		case "rtp", "rtp+rs8m", "rtp+ldpc":
		default:
			return fmt.Errorf("%q uri does not support %q protocol",
				iface, s.Scheme())
		}

	case IfaceAudioRepair:
		switch s.Scheme() {
		case "rs8m", "ldpc":
		default:
			return fmt.Errorf("%q uri does not support %q protocol",
				iface, s.Scheme())
		}

	case IfaceAudioControl:
		switch s.Scheme() {
		case "rtcp":
		default:
			return fmt.Errorf("%q uri does not support %q protocol",
				iface, s.Scheme())
		}
	}

	return nil
}

func (s URI) Scheme() string {
	u, err := url.Parse(string(s))
	if err != nil {
		return ""
	}
	return u.Scheme
}

func (s URI) Fec() Fec {
	u, err := url.Parse(string(s))
	if err != nil {
		return ""
	}
	switch u.Scheme {
	case "rs8m", "rtp+rs8m":
		return FecRs8m
	case "ldpc", "rtp+ldpc":
		return FecLdpc
	}
	return ""
}

func (s URI) User() string {
	u, err := url.Parse(string(s))
	if err != nil {
		return ""
	}
	if u.User == nil {
		return ""
	}
	return u.User.String()
}

func (s URI) Host() string {
	u, err := url.Parse(string(s))
	if err != nil {
		return ""
	}
	return u.Host
}

func (s URI) IP() string {
	h := s.Host()
	if h == "" {
		return ""
	}
	ip, _, err := net.SplitHostPort(h)
	if err != nil {
		return ""
	}
	return ip
}

func (s URI) Port() string {
	h := s.Host()
	if h == "" {
		return ""
	}
	_, port, err := net.SplitHostPort(h)
	if err != nil {
		return ""
	}
	return port
}

func (s URI) Resource() string {
	u, err := url.Parse(string(s))
	if err != nil {
		return ""
	}
	ret := ""
	if path := u.EscapedPath(); path != "" {
		ret += path
	}
	if query := u.RawQuery; query != "" {
		if ret == "" {
			ret = "/"
		}
		ret += "?" + query
	}
	if frag := u.EscapedFragment(); frag != "" {
		if ret == "" {
			ret = "/"
		}
		ret += "#" + frag
	}
	return ret
}
