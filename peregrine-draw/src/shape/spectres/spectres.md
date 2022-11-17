# Spectres

## Introduction

Spectres are the shapes drawn over the top of the display during an interaction to help you with the interaction. For example, the red dotted lines when dragging out a region, the ruler "maypole" and so on. They don't really form a part of the image itself.

Although not part of the genome browser data, it's useful for them to use the facilities used to draw the browser images to draw their shapes. They therefore sit in an awkward half-way house in the pipeline which makes them a little annoying to work with.

## Types of Spectre

* `Ants` -- the dotted red lines drawn around a dragged-out region
* `Stain` -- the greying out of everything not in a dragged-out region
* `Maypole` -- the pole you can create by clicking on the ruler.
* `Compound` -- a spectre made out of multiple other spectres but managed as one

## GLobal classes

The `SpectreManager` maintains a list of spectres currently on the screen. It has means of creating all these spectre types, and when you want to use one you can add it into the manager. (The reason it's a two stage process is that you might want to compound them into a single spectre). After you have added a Spectre to the SPectreManager you get a `SpectreHandle`. Take good care of you SpectreHandle: as soon as you drop it, your spectre will disappear. The idea is that the spectre will form part of some larger process of the interaction and so can simply be stored in the state for that interaction.

## Reactive system

One distinctive thing about spectres is that they frequently move around. Creating and destroying a spectre for each position would be troublesome. To this end, the Reactive system is used. The Reactive system is one way of separating the setters of some value from its effects. In this case, the effects are within the spectre itself. The setters are pieces of code which 