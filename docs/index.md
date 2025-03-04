# rocd

```d2
direction: right

h1: host 1
h2: host 2
h3: host 3

h1.rc1: roc-cast 1
h2.rc2: roc-cast 2
h3.browser: web browser

h1 {
  rc1 -- "rocd 1"
}
h2 {
  rc2 -- "rocd 2"
}
h3 {
  "rocd 3" -- browser
}

h1."rocd 1" -- h2."rocd 2": ZMQ
h2."rocd 2" -- h3."rocd 3": ZMQ
h3."rocd 3" -- h1."rocd 1": ZMQ
```

## Entities

host
:   PC where the roc instance is currently running

roc-cast
:   component used for UI and interaction with real devices on mobile systems

rocd
:   server component used for stream management and auto discovery over network

## rocd

rocd handles devices and streams, and connects ones with others. Its core functions are:

- create, read, update and delete (CRUD) metainfo about system devices (speakers, microphones)
- CRUD metainfo about streaming devices (virtual speakers and microphones passed through network)
- find other rocd instances over network
