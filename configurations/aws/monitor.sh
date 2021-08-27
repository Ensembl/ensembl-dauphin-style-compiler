#!/bin/sh

watch 'tail -6 ~/gb/logs/syslog; echo; du -sh /cache 2>/dev/null ; echo ; docker ps --format "{{.ID}}   {{.Status}}   {{.Image}}"; echo; uptime"'
