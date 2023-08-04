import logging

from model.tracks import Track, Tracks

class Expansions:
    def test(self, step):
        red = int(step[0:2],16)
        green = int(step[2:4],16)
        blue = int(step[4:6],16)
        track = Track(step,program_group="ensembl-webteam/core",program_name='test',program_version=2,scales=[0,100,1])
        track.add_trigger(["track","expand",step])
        track.add_value("red",red)
        track.add_value("green",green)
        track.add_value("blue",blue)
        tracks = Tracks()
        tracks.add_track("test",track)
        return tracks

    def define_variation_track(self, track_id):
        track_summary_name = f"variant-summary-{track_id}"
        track_details_name = f"variant-zoomed-{track_id}"
        track_summary_view = Track(track_id, program_group="ensembl-webteam/core", program_name='variant-summary', program_version=1 ,scales=[6,100,4])
        track_details_view = Track(track_id, program_group="ensembl-webteam/core", program_name='variant-zoomed', program_version=1 ,scales=[1,5,1])

        # FIXME: remove variant-dbsnp below

        # define common settings
        for track in [track_summary_view, track_details_view]:
            track.add_trigger(["track", "expand-variation", track_id])
            track.add_setting("name", ["track","variant-dbsnp","name"])
            track.add_setting("rank", ["track","variant-dbsnp","rank"])
            track.add_value("track_id", track_id)
            track.add_value("track_name", f"Track name for {track_id}")

        # define summary view settings
        track_summary_view.add_value("track_data_id", "variant-dbsnp")

        # define zoomed-in view settings
        track_details_view.add_value("track_data_id", "variant-labels-dbsnp")
        track_details_view.add_setting("label-snv-id", ["track","variant-dbsnp","label-snv-id"])
        track_details_view.add_setting("label-snv-alleles", ["track","variant-dbsnp","label-snv-alleles"])
        track_details_view.add_setting("label-other-id", ["track","variant-dbsnp","label-other-id"])
        track_details_view.add_setting("label-other-alleles", ["track","variant-dbsnp","label-other-alleles"])
        track_details_view.add_setting("show-extents", ["track","variant-dbsnp","show-extents"])

        # register tracks
        tracks = Tracks()
        tracks.add_track(track_summary_name, track_summary_view)
        tracks.add_track(track_details_name, track_details_view)

        return tracks
