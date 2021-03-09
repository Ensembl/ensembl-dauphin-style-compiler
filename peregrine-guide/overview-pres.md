# Peregrine Genome Browser

---

## Requeirements

---

### Existing Site Observations

* Data and code evolve, making maintenance hard
* Displays are slow
* Code is complex and scary

---

### New Site Wishes

* Fast Pan and Zoom
* Replaceable technology

---

### Approach: Start After The Biology

* Capture data as *shapes* not *entities*
* Allow trivial style changes
* Allows old data to be handled well
* Removes our responsibility for science
* Separate constant "style" from variable "data"
* Compression and styling are equivalent

---

## WHat is the Style Language for?

* Implements the "syle" part
* Consumes data, emits "shapes"
* Decompresses data
* Adds colours, heights, labels, etc

---

### What is the Style Language?

* Not a proper programming language
* Based around 1D arrays of simple values
* No branches, no loops
* "Pretend" structures and multi-dimensional arrays

---

### Use of style-language

* As black box
* As black box with custom parameters
* Custom style language
* New primitives

---

### Technology Architecture

* Source file is called `egs`
* command-line *compiler* converts `egs` to `begs`
* `begs` delivered along with data to web browser by backend
* `begs` interpreted by *interpreter*

---

### Code layout

* simple python backend
* confusing pile of rust dependencies
* can probably be cut down but dependency must remain acyclic
