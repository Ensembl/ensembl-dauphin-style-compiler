# Configurations

We support three broad classes of setup.

 * **EBI** -- for live, dstaging and PR at EBI. These use data on local disks and local services as their primary
   data sources.

 * **ad hoc** -- for third party installs, collaborations, etc, where experimental services need to be tested
   and shared without needing the EBI infrastructure.

 * **dev** -- for developers working on thecode.

ad hoc and dev installs are very similar in that their data primarily comes from fixed files and forwarded requests.
On the other hand, the EBI installs use officially produced data sets and internal disks.

# Containers

There are the following docker containers used in these various configurations:

 * hi
 * lo
 * memcached
 * syslog
 * frontend
 * nginx

hi and lo are the backend servers and should always be running. Even when developing the backend, it is simplest to
run them inside containers on your machine. memcached is also vital for everywhere a backend server is run, even in
development.

nginx is vital if you are using HTTP-served datafiles, meaning dev or ad-hoc installs. It is not necessary for EBI
installs. syslog is vital unless you have a good syslog setup of your own, meaning it's not needed for EBI installs,
but is necessary for ad hoc and dev (mostly).

bump-detector detects when files have been updat4ed upstream and makes sure that our-of-date cache entries are no longer used. It does this both for memcached and for nginx. Because of its role for memcached, it is also required everywhere.

frontend is only useful for ad hoc installs. It provides a wrapped build of the genome browser. This is needless
extra effort and potential confusion for devs and of no use for EBI installs. But it is useful in ad hoc installs so
that it can be seen by people who don't have therust tool chain.

|           | EBI | dev | ad-hoc |
|-----------|-----|-----|--------|
| hi        | Y   | Y   | Y      |
| lo        | Y   | Y   | Y      |
| memcached | Y   | Y   | Y      |
| bump-det. | Y   | Y   | Y      |
| nginx     |     | Y   | Y      |
| syslogd   |     | Y   | Y      |
| frontend  |     |     | Y      |

You should use docker-compose to start the containers to wire through volumes, environment variables, etc.

# Architecture

When all the components available are in use, the artchitecture provided here looks like this.

Full setup (eg for dev)

```
+--------------------------+
| browser                  |
+--------------------------+
  ^                  ^   ^
  |                  |   |
static               hi  lo
asssets              |   |
  |                  |   |
+----------+     +----+ +----+
| frontend |     | hi | | lo |
+----------+     +----+ +----+                +-----------+
                  ^ ^    ^ ^---pre-prepared-->| memcached |
+-----------+     |  `---|--------output----->+-----------+
| syslog    |     |      |                          |
+-----------+    data files                   +-------------+
  ^^^      |      |      |                    | bump-detect |
  |||      v     +-----------+<-------------->+-------------+
every-   local   | nginx     |
where     disk   +-----------+<----cache-----> local disk
                  |      |
              remote souce (eg s3)
```

Production setup:
```
+--------------------------+
| browser                  |
+--------------------------+
                     ^   ^
                     |   |
                     hi  lo
                     |   |
                     |   |
                 +----+ +----+
                 | hi | | lo |
                 +----+ +----+                +-----------+
                  ^ ^    ^ ^---pre-prepared-->| memcached |
                  |  `---|--------output----->+-----------+
                  |      |                          |
                 cluster disk                 +-------------+
                                              | bump-detect |
                                              +-------------+
```


If local or nfs disk is in use (eg at EBI) the data files arrows are connected direct to the disk, not to the nginx box.

# Container List

## hi and lo (the main backend servers)

hi and lo are identical servers in terms of code and data. By convention and config, the genome browser contacts
hi for things which are blocking correct display (and so low latency [around 20ms] is required). It contacts lo for data
which it anticipates it will need in future. Latency is less of an issue here. Note that while taking around 100ms is no
major barrier here (even the very occasional 1s request), it is very noticable to the user when the lo service is
unavailable. In practice, most of the data actually displayed to a user first originates from the lo endpoint. So it is
in no way optional.

Internally both containers run on port 3333. This is exposed directly in hi, but translated to 3334 by docker. The
frontend first contacts hi for information at which point it will be told about lo, on the next port.

Requests take a wide range of times and it's hard to predict them in advance. This means that a "next available"
strategy is much better than round-robin or load based strategies, meaning it makes more sense to have a lot of threads
on a server than a lot of instances of a container. In practice this approach doesn't scale indefinitely, however and
as long as there are a substantial number of threads on each container, it is ok to after that scale out on container
number.

Because latency is less of an issue for lo, it need have fewer threads per container and can scale out at the container
level much sooner than hi. In practice 16 threads per container for hi and 4 for lo seems to be the sweet spot.

Logging of both access log and error log is via syslog.

### Envorinment

 * `DEBUG` -- sets log level between WARN and DEBUG among other things.
 * `LOG_HOST` -- syslog hostname
 * `LOG_PORT` -- syslog hostname
 * `THREADS` -- appropriate for the continer use case.
 * `MEMCACHED` -- hostname:port of memcached host
 * `MEMCACHED_PREFIX` -- key prefix to use stop multiple memcached tenants using same keys.

| Variable | EBI | dev | ad hoc |
|----------|-----|-----|--------|
| DEBUG    | 0   | 1   | 0      |
| THREADS  | hi=16 lo=4 | hi=16 lo=4 | hi=16 lo=4 |
| LOG_HOST | per-k8s setup | "syslog" (from docker-compose) | "syslog" (from docker copose) | 
| LOG_PORT | per-k8s setup | 11601 (from docker-compose) |"11601 (from docker compose) |
| MEMCACHED | per-k8s setup | "memcached:11211" (from docker-compose) | "memcached:11211" (from docker-compose) | 
| MEMCACHED_PREFIX | per-k8s setup | "gb" (from docker-compose) | "gb" (from docker-compose) | 

## memcached

memcached is required. Even for development the backend server relies on a cache for any kind of comprehensible experience at all. No particular memcached is required. The container bundled is an unremarkable one. It could easily be replaced with a system-wide resource. dev needs ~512Mb and productiong ~8Gb for a senseible experience. It runs on the standard memcached port 11211.

### Enviroment

 * `MEMCACHED_MB` -- size of cache.

 | Variable       | EBI | dev | ad hoc |
|-----------------|-----|-----|--------|
| MEMCACHED_MB    | 8192   | 512   | 8912 |

## bump detector

The bump detector is a script in a tight loop which runs a few times a minute. It retrieves a tiny file from upstream (`bump`) and sees if its value has changed. If it has it updates the "bump" key in memcached (which is used by the backend-server to create unique memcached keys). It also creates an empty tile in nginx's cache directory which the nginx script will detect and use to clear the cache and restart the nginx server. Note that this nginx cache path must exist (at present) but it can simply be mapped to /tmp etc.

### Environment

 * `LOG_HOST` -- syslog hostname
 * `LOG_PORT` -- syslog hostname
 * `BUMP_URL` -- url (or file path) to check for bump file.
 * `MEMCACHED` -- hostname:port of memcached host
 * `MEMCACHED_PREFIX` -- key prefix to use stop multiple memcached tenants using same keys.

| Variable | EBI | dev | ad hoc |
|----------|-----|-----|--------|
| LOG_HOST | per-k8s setup | "syslog" (from docker-compose) | "syslog" (from docker copose) | 
| LOG_PORT | per-k8s setup | 11601 (from docker-compose) |"11601 (from docker compose) |
| BUMP_URL | disk bump-file location at datafile root | url to s3 bump file | url to s3 bump file |
| MEMCACHED | per-k8s setup | "memcached:11211" (from docker-compose) | "memcached:11211" (from docker-compose) | 
| MEMCACHED_PREFIX | per-k8s setup | "gb" (from docker-compose) | "gb" (from docker-compose) | 

## nginx

nginx, if running, sits between the data-files and the backend servers. Unlike memcached it caches the underlying data sources, not the output. It is not needed if the file access is reasonably fast (eg disk not http). nginx is wrapped in a wrapper script which can restart it should a signal file be created in its cache directory (by bump-detector). This script then empties the cache and restarts.

The cache should be a sizeable portion of disk for temporary files. nginx is not toally expert in managing the space in a timely manner, so a little overhead in terms of space is beneficial.

### Environment

 * `LOG_HOST` -- syslog hostname
 * `LOG_PORT` -- syslog hostname
 * `NGINX_CACHE_SIZE` -- disk cache size (eg `100m`)
 * `FILE_OWNER` -- user to run nginx as (owns cache files)
 * `FILE_GROUP` -- group to run nginx as (owns cache files)
 * `NGINX_CACHE_DIR` -- dir to use for cache

| Variable | EBI | dev | ad hoc |
|----------|-----|-----|--------|
| LOG_HOST | N/A | "syslog" (from docker-compose) | "syslog" (from docker copose) | 
| LOG_PORT | N/A | 11601 (from docker-compose) |"11601 (from docker compose) |
| FILE_OWNER | N/A | eg ubuntu | eg dan |
| FILE_GROUP | N/A | eg ubuntu | eg dan |
| NGINX_CACHE_DIR | N/A | eg /home/ubuntu/cache | eg /home/dan/cache |
| NGINX_CACHE_SIZE | N/A | eg 100m | eg 8000m |

## syslog

All the other containers use syslog to report their logs. On development systems it's more common to use flat files. The syslog container receives syslof entries and writes them to a host file. It's therefore only of use outside production environments. In production environments use your logging stack instead.

### Environment

 * `FILE_OWNER` -- user to run nginx as (owns cache files)
 * `FILE_GROUP` -- group to run nginx as (owns cache files)
 * `LOG_DIR` -- dir for logs

| Variable | EBI | dev | ad hoc |
|----------|-----|-----|--------|
| FILE_OWNER | N/A | eg ubuntu | eg dan |
| FILE_GROUP | N/A | eg ubuntu | eg dan |
| LOG_DIR | N/A | eg /home/ubuntu/logs | eg /home/dan/logs |

## frontend

This container serves static assets for the fronend. It can't build assets so is no use for development and in deployment it is replaced by a proper static asset server. It's therefor reduced to the niche role of a "shop-window" for ad-hoc deployments.

### Environment

No environment variables are used

# Ports

| port (container) | port(exposed) | machine | use |
|------|---------|-----|----|
| 3333 | 3333 | hi | serving frontend |
| 3333 | 3334 | lo | serving frontend |
| 11211 | - | memcached | memcached for backend |
| 11601 | - | syslog | UDP packets for logging |
| 8000 | 8080 | frontend | frontend static assets |
| 80 | - | nginx | nginx cache |
