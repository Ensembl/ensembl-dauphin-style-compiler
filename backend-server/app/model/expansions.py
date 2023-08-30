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
        track_data = variation_tracks_map[track_id]
        track_data_ids = create_variant_track_data_ids(track_id)

        # While for the client, there is only one id for a track,
        # usually there are in fact several tracks associated with an id reported by the client;
        # so here, we are creating artificial (but still unique) ids to register these tracks in a track registry
        track_summary_registry_id = f"variant-summary-{track_id}"
        track_details_registry_id = f"variant-zoomed-{track_id}"

        # here we are declaring different programs to be run at different scales 
        track_summary_view = Track(track_id, program_group="ensembl-webteam/core", program_name='variant-summary', program_version=1 ,scales=[6,100,4])
        track_details_view = Track(track_id, program_group="ensembl-webteam/core", program_name='variant-zoomed', program_version=1 ,scales=[1,5,1])

        # define common settings
        for track in [track_summary_view, track_details_view]:
            track.add_trigger(["track", "expand-variation", track_id]) # to turn a track on/off
            track.add_setting("name", ["track", "expand-variation", track_id, "name"]) # toggle track name on/off
            # track.add_setting("rank", ["track", "expand-variation", track_id, "rank"]) # setting to determine track order
            track.add_value("track_id", track_id) # will be required for defining the track "leaf" in the tree of tracks
            track.add_value("track_name", track_data['track-name']) # inject track name into the track program
            track.add_value("display_order", track_data['display-order']) # temporary, probably

        # define summary view settings
        track_summary_view.add_value("track_data_id", track_data_ids['summary-data-id'])

        # define zoomed-in view settings
        track_details_view.add_value("track_data_id", track_data_ids['zoomed-data-id'])
        track_details_view.add_setting("label-snv-id", ["track", "expand-variation", track_id, "label-snv-id"])
        track_details_view.add_setting("label-snv-alleles", ["track", "expand-variation", track_id, "label-snv-alleles"])
        track_details_view.add_setting("label-other-id", ["track", "expand-variation", track_id, "label-other-id"])
        track_details_view.add_setting("label-other-alleles", ["track", "expand-variation", track_id, "label-other-alleles"])
        track_details_view.add_setting("show-extents", ["track", "expand-variation", track_id, "show-extents"])

        # register tracks
        tracks = Tracks()
        tracks.add_track(track_summary_registry_id, track_summary_view)
        tracks.add_track(track_details_registry_id, track_details_view)

        return tracks


variation_tracks_map = {
    'variant-dbsnp': {
        'track-name': 'Track name for dbsnp',
        'display-order': '1001'
    },
    'variant-1000genomes': {
        'track-name': 'Track name for 1000genomes',
        'display-order': '1000'
    }
}

def create_variant_track_data_ids(track_id):
    """
    So far, the convention we are using for variant track ids
    is as follows:
    - Summary program requests data using the same id as provided in track API (e.g. variant-dbsnp)
        -  VariantSummaryDataHandler adds a suffix "-summary" to the track id
    - Zoomed-in program requests data with an id that has the word "labels" inserted between
        the word "variant" and the rest of the id
    """
    id_parts = track_id.split('-')
    id_parts = [id_parts[0], 'labels', *id_parts[1:]] # injject the word "labels" into the id
    return {
        'zoomed-data-id': '-'.join(id_parts),
        'summary-data-id': track_id
    }
