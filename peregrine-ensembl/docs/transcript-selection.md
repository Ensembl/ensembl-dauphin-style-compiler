# Transcript selection

## Terminology

In this document:

*  `enabled-transcripts` refers to the data passed as the argument `X` passed to `switch(["track","focus","enabled-transcripts"],X)`.

* `shown-transcripts` refers to the data received in the track payload for thefocus track with key `shown-transcripts`.

* `several` is the argument `X` passed to `switch(["track","focus","several"],X)`

## Introduction

This spec is designed to allow the future implementation of lozenge behaviour as well as the current 1/5 transcript switch and the eye icons. For this reason, it might not be clear why certain subtelties are currently required, but it should make snese later. I can always explain if you want to know why.

## Requirements

* The chrome will need a long-term state store (ie across reloads) which is keyed by gene and for each gene can contain either:
    1. a list of transcript ids as strings, or
    2. a distinguished "don't know" value.
    
    This will be called the `transcript store`. Implementation of "don't know" is left as an option to implementor. Could be a distinguished value, could be implemented as mere absence.

* The chrome will need to remember the state of the 1/5 switch long-term (ie across reloads). This will be called the `1/5 status store`.

## Actions

Justifications are added as footnotes (*1, *2, etc) to avoid disrupting flow.

### On load or new focus gene

1. Retrieve the currently stored list of transcripts from transcript store for this gene, if any. If no value is in the store, treat as if "don't know" had been stored there.

2. Send this value as enabled-transcripts (if "don't know", send `null`).

3. Send status from 1/5 status store to several.

4. *DON'T* update the eye icon (*3).

### On an eye icon being clicked on

1. Retrieve the currently stored list of transcripts from transcript store for this gene, if any. If no value is in the store, treat as if "don't know" had been stored there.

2. If the value is "don't know", do nothing. (*1)

3. Otherwise rmove or add your transcript (depending on if the eye is being turned off or on) and send the updated value via enabled-transcripts.

4. *DON'T* store the updated list of values back into the transcript store, and *DON'T* update the eye icon (*2).

### On the 1/5 switch being changed on the focus track

1. Update the 1/5 status store to the correct boolean for the current setting.

2. Send this new setting via the several switch.

3. Send `null` to enabled-transcripts.

4.  *DON'T* store the updated list of values back into the transcript store, and *DON'T* update the eye icon (*3).

### On receiving an update of shown-transcripts

1. Retrieve the current value from transcript store for this gene, if any. If no value is in the store, treat as if "don't know" had been stored there.

2. Update the transcript store to match shown-transcripts and update eye icons. (*4)

3. *ONLY* if this value was *previously* "don't know", send these values *back* to the genome browser in enabled-transcripts (*5).

## Justifications

* `(*1)` The chrome will always receive a list of shown-transcripts in short measure (within <100ms) which means that if the current value is "don't know" then somehow the user clicked as focus gene or several setting was changing and the on-screen indications were probably wrong. This should be very rare. Safest thing to do is drop it.

* `(*2)` The actual transcripts shown can vary from what the chrome imagines depending on the lozenge setting, etc. So it needs to only record state and UI sessings on receiving from shown-transcripts.

* `(*3)` See (*2). Also, whenever enabled-transcripts is set to null it will receive a shown-transcripts in short measure anyway.

* `(*4)` Only this message contains the _actual_ transcripts shown and so this exclusively should be used to update the UI and the state.

* `(*5)` Only doing it when "don't know" prevents loops.
