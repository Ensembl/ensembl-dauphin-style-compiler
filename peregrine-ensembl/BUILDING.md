Rust Build type
---------------

Flags to use for rust release

* *dev* -- faster to build; more detailed debugging information. Slower when running; larger binaries.
* *release* -- faster when running; smaller binaries. Slower to build; little debugging information.

*release* is probably a sensible default if you don't care.

Console verbosity
-----------------

How noisty to be on the console.log in the running browser

* *default* -- a few useful messages on startup
* *quiet* -- absolute minimum info to console
* *noisy* -- incldue debug information on console

*default* is a sensible default.

ensembl-genome-browser code
---------------------------

Do you have your own copy of the ensembl-gnome-borwser repo which you want to use? (eg because you are testing a
bugfix in it)?

* *github* -- just use the one on github
* *local* -- I have a local one

*github* is a sensible default.

cache ensembl-client for around one day?
----------------------------------------

Cacheing the ensembl-client repo speeds up builds. If you don't expect any pertinent changes say yes.

* *yes* -- cache if less than around a day old
* *no* -- always pull

*yes* is a sensible default.

clear build caches
------------------

Building uses some docker tricks to cache npm, cargo packages etc. This should be transparent, but if you suspect
some stale code, clear the caches at the expense of a much longer build.

* *yes* -- clear caches
* *no* -- don't clear caches.

*no* is a sensible default.

backend
-------

Which backend to use?

* *proxy* -- use the proxy server (which forwards to EBI staging)
* *local* -- use a backend server running on your local machine
* *s3* -- use the amazon backend server
* *staging* -- use the EBI staging server.

At present the proxy always forwards  to staging, so staging and proxy are equivalent (except staging is cross-domain).
This won't always be the case, though, so *proxy* is a better default.

buildkit console format
-----------------------

Buildkit has a fancy console format which is normally very useful as an overview but you can lose things if you are
trying to track down an issue.

* *fancy* -- use the fancy format
* *plain* -- use a normal console format

*fancy* is a sensible default.

port number
-----------

What port do you want to server the things you've just built on? 0 = don't run a server, just exit.
