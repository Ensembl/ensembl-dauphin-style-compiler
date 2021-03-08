# Technology Architecture

This document is incomplete.

## Overview

When producing the data:

* You find or write *an* `egs` *file* in an editor in the style language, which details:
    * where the data comes from,
    * how data is compressed and
    * what it should look like;
    * You produce the data;
* A command-line tool called the *style-compiler* turns the `egs` file into *a* `begs` *file*.
* The `begs` file sits on the backend server and is served to the frontend.
* The `style-interpreter` in the frontend (in the browser) interprets the begs file.

```
+--------------+  +----------+
| Data Source  |  | egs file |    <- Data Creator
+--------------+  +----------+
       |               |
       |               v
       |          +----------------+
       |          | style compiler | <- command line
       |          +----------------+
       |               |
       |               v
       |          +-----------+
       |          | begs file |    <- Backend Service
       |          +-----------+
       |               |
       v               v
+---------------------------------+
| genome browser backend service  |
+---------------------------------+
                ||
                \/                 <- THE INTERNET
+---------------------------------+
| style interpreter               | (in the browser)
+---------------------------------+
                 |
                 v
+---------------------------------+
| browser drawing code            | (in the browser)
+---------------------------------+
```

The style compiler does the 'heavy-lifting' in making `egs` code efficiently executable and is large and complex, so it sits on the command line. In theory therecould be a "just-in-time" facility in the genome browser backend, but I don't think that will be all that useful in practice.

So the code is divided into a number parts:
* the style-compiler rust which compiles to a command line executable.
* the style-interpreter rust which compiles to web-assembly.
* the webgl drawing etc rust stuff which compiles to web-assembly.
* the python code running the backend service.

The style-compiler and style-interpreter obviously share a great deal of common code. All non-core commands sit in *libraries*, which have a part for the compiler and a part for the interpreter.

Code is organised into multiple crates to help with modularity.
