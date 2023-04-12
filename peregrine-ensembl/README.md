# Ensembl Genome Browser
This module builds a webassembly package of Ensembl genome browser that can be consumed by other projects.

## Api
The genome browser javascript module produced by `wasm-pack` is published as `@ensembl/ensembl-genome-browser` npm package to an EBI package registry. It has the following api for interacting with the genome browser.

### Loading and initialization script
The default export from the module is an asynchronous initialization script. Run it before calling any methods on the genome browser

```js
import initializeGenomeBrowser from '@ensembl/ensembl-genome-browser';

initializeGenomeBrowser.then(/* now the genome browser should be ready */)
```

### GenomeBrowser class
A named `GenomeBrowser` export is a class through which you can interact with the genome browser. It has the following instance methods:

- `.go(config: GenomeBrowserConfig)` — a method to start the genome browser. It accepts a config of the following shape:

```ts
type GenomeBrowserConfig = {
  backend_url: string;
  target_element?: HTMLElement; // container element into which the genome browser will render its canvas
  target_element_id?: string; // alternative (less preferred) way to make the genome browser aware of the container element it needs to use
};
```
- `.set_stick(stick: string)` — enables a track, or, in genome browser's jargon, a stick. The identifier of a stick typically consists of a genome id and a region (e.g. chromosome) name, separated by a colon.

Example:

```js
genome_browser.set_stick("a7335667-93e7-11ec-a39d-005056b38ce3:13");
```

- `.jump(jump_identifier: string)`

The `jump` method makes the genome browser jump to a particular feature. Jump identifiers are strings that typically consist of track_id, genome_id, and feature_id, separated by a colon.

The `jump` method can be followed by a call to the `wait` method to let the genome browser animate its journey ot the location of the feature to which it is jumping.

Example:

```js
genome_browser.jump("focus:homo_sapiens_GCA_000001405_28:ENSG00000073910");
genomeBrowser.wait();
```

- `.goto(start: number, end: number)`

The `goto` method tells the genome browser to navigate to the specified location on the "stick" (i.e. genomic region). Make sure that the "stick" has been set before calling the `goto` method (see the `set_stick` method above).

- `.switch(path_to_setting: Array<string>, parameters: unknown)`

The `switch` method is used to turn tracks, as well as the settings of a given track, on or off. The path to a setting, as well as the parameters that a setting takes, depend on the configuration of the genome browser styles file.

Examples:

```js
// Turn on the focus gene track.
// Notice that both calls to the `switch` method are necessary; and that the parameter is a JSON object
genomeBrowser.switch(['track', 'focus'], true);
genomeBrowser.switch(['track', 'focus', 'item', 'gene'], {
  genome_id: genomeId,
  item_id: focusId
});


// Turn on specific transcripts in the focus gene track. The parameter is an array of transcript ids.
genomeBrowser.switch(
  ['track', 'focus', 'enabled-transcripts'],
  ['ENST00000544455.6', 'ENST00000680887.1', 'ENST00000530893.6', 'ENST00000614259.2', 'ENST00000665585.1']
);
```

- `.receive_message(callback: (message_name: string, payload: any) => void)`

This is the method for subscribing to messages sent by the genome browser (see below).

### Genome browser messages

- Current location message. Message name: `current_position`. Message payload:

```js
{
  stick: string; // formatted as "<genome_id>:<region_name>"
  start: number;
  end: number;
}
```

- Target location message. Message name: `target_position`. Message payload: same as for current location message. It differs from the current location message in that it is sent after the genome browser has arrived at the position it was told to go, or when the user has released the canvas. This message is used for UI updates that should happen at a lesser frequency than the ones caused by the current location messages.

- Track summary message. Message name: `track_summary`. Message payload:

```ts
{
  summary: Array<{
    'switch-id': string;
    offset: number;
    height: number;
    ... // plus optional custom fields defined in genome browser style files 
  }>
}
```

- Error message. Message name: `error`. The payload contains genome browser error messages.




## Publishing the module
See [publish/README.md](./publish/README.md) for instructions on how to build and publish the package.
