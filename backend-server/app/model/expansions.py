import logging

from model.tracks import Track, Tracks

class Expansions:
    def test(self, step):
        red = int(step[0:2],16)
        green = int(step[2:4],16)
        blue = int(step[4:6],16)
        track = Track(step,program='test',scales=[0,100,1])
        track.add_trigger(["track","expand",step])
        track.add_value("red",red)
        track.add_value("green",green)
        track.add_value("blue",blue)
        tracks = Tracks()
        tracks.add_track("test",track)
        return tracks
