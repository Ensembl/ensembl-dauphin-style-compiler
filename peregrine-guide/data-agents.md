# Agents

## Introduction

Peregrine-data is based around agents. These are essentially just memoized funtioncal queries. They are typically
also async.

The most important agents in peregrine-data are based around generating shapes (inside a ShapeOutput) based upon the
selected programs and the current region.

## LaneStore

The current requested region is a Lane: it comprises stick, position, zoom, TrackConfig for the lane. The TrackConfig
includes the track_name (a temporary hack) the program name and the config for that program.

The lane store allows you to specify a Lane and recevie the corresponding ShapeOutput. Hopefully it will be cached.

If the shape data is not cached, we will need to generate new dat. We gain efficeincy by not rerunning at every scale,
instead we can run at a few scales where we retain sufficient accuracy and then filter. This is also managed by the
lane store.

First the lane_scaler agent is called which maps from the wanted lane to a good lane to use as the basis of the program.
The actual running of the program is then delegated to the ShapeProgramRunAgent with the wanted lane. Once this is
complete, the results are filtered and scaled for the original lane.

## ShapeProgramRunAgent

ShapeProgramRunAgent has its own cache of shapes (as we expect more hits here as fewer shape lists cover more data).
However, should there be a cache miss, then the relevant dauphin program is run and the cache populated.
