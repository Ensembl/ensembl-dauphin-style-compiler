# Using tangle

## Introduction

Each tangle comprises one or more sources connected to one or more drains. In the middle is a process which transfers from source to drain, applying some transformation in the process, known as a tangle.

Each drain expects to receive a key name and a series of bytes. Each source provides information in some structure per row, the type of which is understood by that tangle.

Tangles come in a number of types. These tangletypes are stored in the tangle-registry which is initialised in code at startup. The tangle chosen from the tangle-registry is determined by the keys in the config (and potentially the values of those keys).

A tangle identifies some keys in the config as being used to specify sources. Sources are chosen from the available source-types based on the values of those fields. Each source is of a certain type and each instance of the type is configurable by a "main string" and an essentially arbitrary additional dict.

The source's type is chosen by the .type subkey and the main string is detemined by the .field subkey. Other keys are passed in the additional dict. Should a string be used instead of such a dict of subkeys, the default source is used and with an empty additional dict. This is very common as the default type is usually sensible and additional config optional.

Most tangle keys are passed to the tangle to use in configuration as is wished on a type-specificbasis.

However, some keys are used to control operation of serialisation itself and are examined before (or instead of!) the tangle itself.

* `input` -- the input to use for this tangle if not the default. Very rarely needed.

* `condition` -- tangles with this key are only called at all if the condition is perent in the run. This allows subsets of information to be delievered in different circumstances.

* `first` -- the value of this field is a source. For each value of this source only the
first is emitted.

## Common keys

Although tangle types can support other keys as they wish, they try to adhere to conventions to make things consistent. In the Tangle types section below, if a key is mentioned as supported with no further info, it is assumed to be as per the definition in this section.

* `name` -- string value naming the drain. Where a tangle produces multiple drains, the name given here is used as the "base" for the name along with suffixes for each (by default). If this key is not provided, the tangle name is used instead.

* `X_name` -- where multiple drains are created usually they are composed by standard rules from `name` as described above. Hoever, individual drains can be named with these keys.

* `uncompressed` -- do not perform the final byte compression step. Useful for debugging, etc.

## Tangle Types

You can add to the tangle-registry, but the following are registered by default and are the most common.

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
