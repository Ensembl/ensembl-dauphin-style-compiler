# Backends

There are a number of distinct terms which need to be kept in mind when talking about "the backend". In general the term is avoided as it is ambiguous.

## Distinctions

* `domain` -- a domain is a _method_ of accessing a backend. For example, the `network` domain uses http/https and cbor to get its data; the `jsapi` domain uses javascript callbacks. A domain is basically a registered body of code.

* `backend namespace` -- a backend namespace is a key which serves to represent each independent endpoint which the frontend knows about.

* `accessor` -- an accessor is a string which may be recognised by any of the registered domains as refering to a backend namespace which it can access. In general accessors are things like URLs in that they provide some means of accessing a backend namespace.

* `mini-boot` -- to use an acessor, the backend namespace needs to be mini-booted. mini-booting comprises calling the boot method on that backend namespace via its accessor. In its response will be various assets and sidecars including programs, tracks, expansions, etc. It is called mini-booting because it doesn't necessarily happen at application boot time (though it usually does) and comprises a small subset of the overall boot process of the browser.

* `essentials directory` -- when a backend namespace is mini-booted, it may provide an essentials directory as a part of its mini-boot response. This notifies the frontend of other necessary backend namespaces, and accessors for them. These will also be mini-booted immediately (which is why they are called "essential"). Note that directory enquiries (see later) is not consulted as a part of this process, which is why the essentials directory contains both the backend namespace and the accesssor.

* `directory enquiries` -- this is an endpoint in an already mini-booted backend namespace which can convert another backend namespace into an accessor. *After* boot, when a new backend namespace is encountered without a known accessor, directory enquiries is called on *all* known backend namespaces to look up the accessor.

* `root accessors` -- a set of accessors known as "the root accessors" are supplied at boot time. Once everything is ready to use them, the root accessors are mini-booted into a set of backend namespaces, and transitively through all essentials directories. Only when this is done *for the whole application* during boot are *any* backends allowed to send anything other than boot messages and their consequences. 

## Application booting

1. The application is booted with the root accessors list.
2. Each domain is consulted in turn, to mini-boot each root accessor.
3. The process continues transitively through the essentials directory.
4. The boot process is unlocked.

## What is locked?

1. Switch operations are stalled until booted. These could trigger tracks (possibly via expansions) and these may not be ready prior to boot. In practice this means there will be no track or expansion requests issued, but the process must be stopped prior to this, allowing no setting at all. As no program can be run, there will also be no data or metric requests.

2. jump operations are stalled until booted, as these contact all endpoints.

3. stick operations are stalled until booted, as these contact all endpoints.

4. no metrics are issued as these contact various endppoints.

Note that booting can lead to other request types (only program requests at the moment, but potentially more in future), so the request queues themselves are not locked.
