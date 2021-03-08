# Motivation of Approach

This document does *not* describe the technology choices, but focuses instead on *data*-lifetime and lifecycle-management, and issues around *data*-representation. The technologies used to implement this architecture are secondary to these concerns and are described elsewhere. Those technologies (such as platform, storage format, retrieval method, etc) are designed to all be incrementally replaceable ifneeded. In the end the new genome browser will probably be a "Trigger's Broom" or "Theseus' Ship" with every part replaced, and if we can manage to achieve that, the design here will have been a success.

## Experience With Existing Ensembl Site

We begin with a description of the factors which motivated the architecture decribed here. The long period of existnce of ensembl has challenged the existing implementation in a number of ways. In particular:

* data and code evolve making both hard to maintain;
* displays are slow to draw (needing lots of resources).
* complex operations are performed at runtime (needing complex code).

### Data and Code Drift

1. Data-models and data-sets have evolved along with scientific understanding and experimental techniques. But the scientific need for reproducibility and supporting-data means that data can often not simply be changed migrated. The result is an awkward compromise of a tangle of archives running old versions of code and data. As well as confusing users this is problematic for maintenance, particularly in terms of security and the growth of the number of installed services.

2. As well as the historical aspect, at any given time, scientists across the world may not agree on the correct model for data or what should or should not be displayed, and with what prominence. This is effectively the synchronic equivalent of the historic issues of the first point.

While we may be at the stage of "a gene is a gene",  consider areas such as:

* histone modification
* binding factor sites
* cryptic start sites
* exon skipping
* ribosomal frame-shifts
* selenocystines, and other novel amino-acids,

and many other areas where understanding, representation, and degree of elaboration is even now in a state of flux across projects.

These particualr issues can be resolved by modifying a given model, but we cannot easily predict where new areas of uncertainty, annotation, and elaboration will take place in the future.

### Slow computation of display

The current site can be extremely slow to display images. Even taking into account its now archaic choice of technologies, it is slower than we might expect.

While it's easy to cast sidelong glances at old technologies and poor coding, the real reason for the slowness has become clear with experience and experiment. 

Quite reasonably, ensembl uses a data-model which is close to underlying biological reality. Unfortunately, what needs to be displayed on screen can have a complex relationship with the unerlying data, if not in algorithmic terms then in terms of data volume.

This is particularly the case for very "zoomed out" views where a single pixel can summarize the effects of thousands or millions of individual elements. We can be confident that this is the underlying cause of this slowness as a very hacky means of speeding this up (precache) was bolted onto the existing site and speeds up display by a factor of 5 to 10, despite being suboptimal in many ways. This is a particularly acute issue for variant displays andother displays which vary base-by-base.

### Complex code at render time

While some slowness is caused by large data volume, other displayed information doesn't use particularly large data-sets, but does use complex algorithms to determine what to display. For example, this is a particularly acute issue for compara displays and in regions of assembly complexity, both of which can involve complex interactions with the assembly mapper. The problem which arises here is not so much speed as the need to preserve old code and an unwillingness to improve or change the algorithm (in case it subtly changes existing information). Even understanding the process is now at best limited to a small number of team members.

## Desires for the new site

In addition to the issues faced by the old site it was seen as desirable to:

* allow fast pan and zoom
* allow techology-independence

### Fast pan and zoom

It was considered desirable that the new browser could efficiently pan and zoom through many scales. On the current site, especially given the slow load times, users are understandably extremely reluctant to scroll. The ability to pan and scroll fluidly and quickly (without fear of some delay which makes them wish they hadn't tried) was seen as highly-desirable. Doing this, of course, would amplify any concerns over data volume and computation complexity.

### Technology Independence

One of the reasons the current site is locked so closely into very old technology is that much of the code is not easily modifiable by a programmer with general skills no matter how competent, because it embodies scientific concepts and bodies of knowledge which are essentially from outside their field.

Much of the code is beyond even a general biologist or bioinformatician and in some cases there are very few people in the world who could inderstand the subtleties of the strategies adopted. Areas where this is the case include much of the assembly mapper as well as portions of the compara, variation, and regulation code.

This complexity minimises the willingness to migrate between technologies, given the typical timescales of the web. It was therefore seen as desirable in the new code-base to produce a system where no single component was irreplacable, was disposible, and easily migrated between technologies or extended.

## Broad Outline of General Approach

### Using Shape-Model Data to Drive The Genome Browser

The main strategy taken to address the issues above was to store data in the form of a *Shape Model*. A shape model is essentially about what is to be drawn on the screen rather than the underlying data. When data is first generated by bioinformaticians, it is processed such that this shape model is the persisted format.

A data creator would likely use this shape model as the persistent format for the reasons given below. If, however, the data creator chooses to produce such data on the fly from biological data, they may do so, the responsibility for the scientific programming issues described above then falling upon them to address through their own service.

A shape model is an abstract description of what is to be placed on the screen. Whatever the underlying scientific model, for a genome browser everything comes down to drawing something on the screen. The contents of ZMenus (popups which appear due to screen clicks) are also in the scope of this model.

A shape model amounts to objects corresponding to rectangles, triangles, colours, labels, and so on, and their relative positions. It is more abstract than an image tile, but still represents data as a series of visual or interacitve objects rather than biological data. The fundamental visual language of a genome browser is likely to evolve at a much slower rate than the biological model, and even when it does change is likely to be augmentative, new language being added, rather than old language being taken away. For exmaple, it is perhaps possible that icons, polygons, or strange loopy lines be added, but very unlikely that the line or rectangle be removed. Augmentative changes are much more eaily made backwards-compatible than non-augmentative ones.

When data is stored as a shape model, it can be "archived" and displayed indefinitely, surviving many changes to biological data model. It does not require agreement between providers as to the correct representation to use. (We might want to enforce this ourselves in many cases, but we shouldn't deliberately restrict our technology as a blunt weapon of policy: "computer says no".

Visual data can also overcome the problems of excessive volume or complexity of computation at each position, and removes the depdence on code which is not easily changed by a general programmer, by being executed after the biological concerns are complete.

### Using Shape Models and not Images

A disadvantage of using a purely image-based approach (such as image tiles) is that they cannot accommodate perfectly reasonable, superficial future requests for example to recolour, reorient, or reshape elements, or move them relative to other features at the same locus. Using a shape-model allows these operations, particularly if these shapes have identification tags (such as names) to assist in such operations. Making a blue element green, or a triangle thinner that another might be squeezed in, and other such purely visual changes, should be accommodated by our representation even in retrospect.

We distinguish *image data* (for example the start and end location of a shape) from *image style* its colour, shape, and so on. Such a distinction is not hard-and-fast and will be a judgement call dependign on the shape. But for most cases the distinction will be clear and will likely allow the vast majority of requested changes to be entirely in the realm of *image style*.

### Issues of Data Volume

Using an image-based model raises concerns over data volume. A visual model of an element is likely much more verbose than a biological model. A series of rectangles representign variants, for example, will likely all be the same height and tend to be rather short (perhaps typically 1bp). Their colours will be drawn from a small set. However to represent colours and heights generally enough that the same methods may be used on other tracks without the unnecessary duplication of primitive operations requires careful consideration of encoding.

In addition, if the demands for pan and zoom are to be more onerous in the new system, even more attention needs to be paid to data encoding. For example, it is likely that the text "positive strand" or "negative strand" will be repeated thousands of times in some data sets despite representing a single bit of information. It is trivial to devise a coding which reduces this to one bit per element (rather than 15 bytes, a factor of 120), it is difficult to do so without loss of generality.

A key innovation in the new architecture is that *styling and data-compression are the same problem*. Compressed, repetitive information is the hallmark of a uniform style and both can be addressed by some style language to overlay the image data. The style language can represent missing elements and various decompression schemes used to default and extract values. And update of this language can allow restyling without altering the underlying data.

### Summary

In summary, the design involves using a *Shape Model* representation of data, after biological processing has taken place. This model is divided into two parts.

* An *image data* part comprises core positional attributes of the data such as start and stop co-ordinates, various flags denoting strand, etc, (though their biological significance is not encoded except by convention in this part).

* An *image style* part which reconstructs this data into its visual consequences including supplying fixed values, decompressing compressed values, decoding flags into various strings, and so on.

The *image data* and *image style* are sent separately over the wire. The style is sent (approximately) once per track type, the data for each region. The two are combined in the browser (which, in the process, greatly expands the data).

Should the style of the track need to be changed, the image style element can be modified. This approach addresses the issues raised in the previous sections.
