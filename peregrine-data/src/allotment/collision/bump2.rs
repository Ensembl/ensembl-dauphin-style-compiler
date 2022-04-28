/* It creates a much more complex algorithm to incrementally bump than to redo it when a
 * carriage is added, taking care to retain existing bumpings. The result would probably
 * be slower and certainly more buggy than just doing it all again for the three
 * carriages.
 *
 * We keep track of the reqeusts in each carriage, but actually use a hot cache of the
 * assignments. When this cache gets too large it is repopulated with the still extant
 * requests.
 * 
 * Requests take the form of a start, end, and height. Results are the same data plus an
 * offset.
 */

