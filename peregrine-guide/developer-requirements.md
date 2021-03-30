
# Developer requirements

This documents is incomplete.  

This document is intended to help you in getting the genome browser setup and running on your local machine for debugging and development purposes.

## Global requirements

### Python version 3

Python is an interpreted, high-level and general-purpose programming language. Python's design philosophy emphasizes code readability with its notable use of significant indentation.

Installation guide: https://www.python.org/downloads/

Please note that you might need to install the required python packages using the command `python3` rather than `python` if you also have Python 2 installed.

### Pip

`pip` is the package installer for Python. 

Installation guide: https://pip.pypa.io/en/stable/installing/

### Rustc

`rustc` is the compiler for the Rust programming language, provided by the project itself. Compilers take your source code and produce binary code, either as a library or executable.

Installation guide: https://www.rust-lang.org/tools/install

More info: https://doc.rust-lang.org/rustc/what-is-rustc.html

`Cargo` is the Rust package manager. Cargo downloads your Rust package's dependencies, compiles your packages, makes distributable packages, and uploads them to crates.io, the Rust community’s package registry.

Cargo usually gets shipped along with `rustc` so we may not have to install it separately.


## Backend-server requirements

### uvicorn

Uvicorn is a lightning-fast ASGI server implementation, using uvloop and httptools.

Installation guide and more info: https://www.uvicorn.org/

  

### wasm-pack

`wasm-pack` helps you build rust-generated WebAssembly packages that you could publish to the npm registry, or otherwise use alongside any javascript packages in workflows that you already use, such as webpack.

Installation guide: https://rustwasm.github.io/wasm-pack/installer/

More: https://github.com/rustwasm/wasm-pack

### fastapi

FastAPI is a modern, fast (high-performance), web framework for building APIs with Python 3.6+ based on standard Python type hints.

Installation command: $ `pip install fastapi` 

More info: https://pypi.org/project/fastapi/
  

### loguru

Loguru is a library which aims to bring enjoyable logging in Python.

Installation command: $ `pip install loguru`

More info: https://pypi.org/project/loguru/
  

### pyyaml

YAML is a data serialization format designed for human readability and interaction with scripting languages. PyYAML is a YAML parser and emitter for Python.

Installation guide: $ `pip install pyyaml`

  

### toml

A Python library for parsing and creating TOML.

Installation guide: $ `pip install toml`

### cbor2

This library provides encoding and decoding for the Concise Binary Object Representation (CBOR) (RFC 7049) serialization format.

Installation command: $ `pip install cbor2`

More info: https://cbor2.readthedocs.io/en/latest/


### pyBigWig

A package for accessing bigWig files using libBigWig

Installation command: $ `pip install pyBigWig`

### requests

HTTP library for Python.

Installation command: $ `pip install requests`
  

### Data files path:

The path is configured in core/config.py in a variable called DATA_FILES where it's pulled in from .env
  

**INTERNAL ONLY**

Download an unpack the following file inside the data files directory (./example-data/data/) before starting the backend server:

`/homes/dan/datafile-upload.tar.gz`

Copy from `ebi-cli-001`:

$ `rsync -aWv your-username@ebi-cli-001:/homes/dan/datafile-upload.tar.gz backend-server/example_data/data/`

Unpack by running:

$ `tar -xf datafile-upload.tar.gz`

Starting the backend server:

$ `./build.sh`

  
# Peregrine-standalone

The standalone application can be started by running the following:

$ `cargo build`

$ `./build.sh`

The server will be started and run on port `8000` on your local machine.

The port can be changed by updating it in the file `./build.sh`