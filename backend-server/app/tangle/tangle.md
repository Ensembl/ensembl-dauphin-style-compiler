# Using tangle

## Introduction

Each tangle.comprises one or more sources connected to one or more drains. In the middle is a
process which transfers from source to drain, applying some transformation in the process.
known as a tangle.

Each drain expects to receive a key name and a series of bytes.
Each source provides "some" information per row, of a type understood by the tanlge.

Tangles come in a number of types. These tangletypes are stored in the tangle-registry which
is initialised in code at startup. The tangle chosen from the tangleregistry is determined
by the keys in the config (and potentially the values of those keys).

A tangle identifies the values of some keys as sources. Sources are chosen from the
available source types based on the value of those fields. A source is configured with a string and
string-to-toml dictionary. sourcetype is chosen by the .type key and the main string is detemined by the .field subkey. Other keys are passed in the dict. Should a
string be used instead of subkeys, the default source is used with an empty dict.

Most keys are passed to the tangle to use in configuration as is wished. However, some keys
are used to control operation of serialisation.

* input -- the input to use

* condition -- these tangles are only called at all if the condition is perent in the run. This
allows subsets of information to be delievered in different circumstances.

* first -- the value of this field is a source. For each value of this source only the
first is emitted.

## Common keys

Although tangle types can support other keys as they wish, they try to adhere to conventions to make things consistent. In the Tangle types section below, if a key is mentioned as supported with no further info, it is assumed to be as per the definition in this section.

* `name` -- string value naming the drain. Where a tangle produces multiple drains the given name is used as the "base" for the name. If this key is not provided, the tangle name is used instead.

* `X_name` -- where multiple drains are created usually they are composed by standard rules from `name`. Hoever, individual drains can be named with these keys.

* `uncompressed` -- do not perform final byte compression. Useful for debugging, etc.

## Tangle Types

You can add to the tangleregistry, but the following are registered by default and are the most common.

### string tangle

*Triggered by:* `string` key, with source value. Can also be array of source values where first truthy value is used for each row.

*Expects:* string or iterable of strings.

Creates single drain encoding data as list of strings. No special assumptions are made about string contents to improve compression.

*Additional keys:* `name`, `uncompressed`.

### classify tangle

*Triggered by:* `classify` key, with source value.

*Expects:* string or iterable of strings.

Creates two drains, one of strings and one of numbers, named `X_keys`, `X_names` respectively. Encodes values from small sets of strings efficiently by coding up the values used and then indexing them. Used for biotypes, designations, etc.

*Additional keys:* `name`, `keys_name`, `values_name`.

### interval tangle

*Triggered by*: `start` key, with source value and one of `end` or `length` key with source value.

*Expects*: single integer or equal-length iterables of integers from both given sources. If delta is upplied same criteria. Delta may also be a single value if other inputs are arrays.

Creates two drains, each of numbers, named `X_starts` and `X_lengths` coding start and length of interval with the assumption that length will tend to be positive and monotonically decreasing from zero and start will tend to be increasing. If `end` is supplied, rather than `length`, length is first calculated by difference. 

*Additional keys:* `name`, `starts_name`, `lengths_name`, `delta` (value to add to all starts and ends).

## number tangle

*Triggered by*: `number` key (source valued).

*Expects* single integer representing the number of some quantity. Can also take iterable, in which case the number of values in the iterable is used.

Creates a stream containing integers which tend to be the _number_ of some entities (ie tending to be positive and monotonically decreasing from zero).

*Additional keys:* `name`, `delta` if true encodes as differences. `positive` number is guaranteed to be >= 0.


