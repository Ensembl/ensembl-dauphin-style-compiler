# Sticks

## Introduction

The genome needs auxilliary information about sticks (broadly, chromosomes). These are stored in the `StickStore`.

If a stick is not present in the stick store, the `try_lookup` method is called for the id on the `StickAuthorityStore`.
The `StickAuthoritySotre` then calls `try_lookup` on each stick authority. Each stick authority has a _resolution
program_.  These programs call the `add_stick` dauphin function which populates the stick store via `add`.

Once the program is complete, either the stick store will contain the stick or not. If it does not then the memoizing
is completed by None. This technique relies on the stick store geing complete, ie not a cache.