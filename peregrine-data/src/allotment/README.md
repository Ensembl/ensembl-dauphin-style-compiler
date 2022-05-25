The top level object is Universe.

Within Universe are a number of LinearRequestGroups. A LinearRequestGroup is parameterised in a 
LinearAllotmentRequestCreatorImpl type which it takes as an argument during construction, and is known as the
"group's creator". When a request is made it is passed to the group's creator's make method. This gets a
LinearGroupEntry in response, which is stored in the LinearRequestGroup. THe LinearGroup entry also has its
make_request method called which returns an AllotmentRequest to the requestor.

During later allocation, the LinearGroupEntry's make() method is called with various supporting metadata matinained
by the group. 

There are two implementations of LinearAllotmentRequestCreatorImpl -- OffsetAllotmentRequestCreator and
MainTrackRequestCreator -- both relatively simple, doing little more than creating relevant implementaions of
LinearGroupEntry -- OffsetAllotmentRequest and MainTrackRequest respectively.

Both OffsetAllotmentRequest and MainTrackRequest use BaseAllotmentRequest which is, in turn parameterised by an
implementation of AllotmentImpl. For the BaseTrackRequest in both OffsetAllotmentRequest and MainTrackRequest that's 
OffsetAllotment. Allotment wraps AllotmentImpl.

When LinearRequestGroup first gets a request to make a request, it looks to see if it has an LinearGroupEntry with a
matching base name. If it doesn't the croup's creator's make() method is called to make one. make_request is then
called on this entry.

