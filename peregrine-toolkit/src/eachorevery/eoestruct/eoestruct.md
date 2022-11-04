# EoEStruct

## Overview

The fundamental structure for data in the genome browser is the "EachOrEvery" (EoE). Every EachOrEvery is one of two things:

1. a finite sequence of values, resembling an array ("each")
2. a conceptually-infinitely long repeating sequence of a single value ("every").

These cannot be nested, and so are flat, sequence data-structures representing, say, a list of start co-ordinates, biotypes, colours, etc. A group of EoEs can be iterated through together. The iteration terminates when the finite EoEs end (of which there must be at least one, and all must be the same length).

The question arises as to map this to more conventional data-structures such as represented in JSON, transforming in "both directions", ie:

1. how do we take a more conventional data source and read and manipulate it as EoEs?
2. how do we combine EoEs and use them to create a more conventional data source?

The answer is: with EoEStruct.

EoEStruct is well covered by very comprehensive unit tests.

The most important type in EoEStruct is `StructBuilt`. This compactly represents conventionally-structured data. This document is divided into two parts:

1. Getting EoEs out of conventional data
2. Generating more conventional datafrom EoEs

## Getting EoEs out of conventional data

Sometimes you have some conventional data (in a `StructBuilt`) and you want to extract some EoEs for inside your style program.

To extract such data you need to specify the path (an array of strings describing which bit of the data you want) and pass it to `struct_select`. You will get a vector of optional `StructConst`s out. A `StructConst` is one of the leaf types in the JSON data-model (number, boolean, string, null). It is optional because it may be missing for any given object. For example, you may have a structure comprising an array of objects and the key you want simply not be present in some of those objects. You can then (hopefully!) build this result into your EoE.

What remains is the syntax for a path.

### Path syntax

A path is an array of strings. Those strings can be

1. a string representing a key in an object;
2. a stringified integer representing the index in an array;
3. `*` representing all instances in an array.

For example, if you want all the start co-ordinates, `[w0,w1,...]` from a structure like this:

```
[
    { "start": w0, "end", x0, "height": y0, "colour": z0 },
    { "start": w1, "end", x1, "height": y1, "colour": z1 }
]
```

then the path is `["*","start"]`.

But for the end-coordinate `[x0,x1,...]` of data like this:

```
{
    "objects": [
        { "range": [w0,x0] },
        { "range": [w1,x1] },
        { "range": [w2,x2] },
        { "range": [w3,x3] },
        { "range": [w4,x4] }
    ]
}
```

then the path is `["objects","*","1"]`.

That's all there is to it! (For late values, see the later section in this document).

## Generating more conventional datafrom EoEs

Sometimes you have some EoEs and want conventional data to serialise. `StructBuilt`s can be serialised into json, but how do you get one for your data and template? You build a `StructTemplate` and then call `build()` on it. This is much more fiddly than the other way around because you need to specify how you want the data strcutured.

Say you have a bunch of EoEs representing some data, for example, one contains start coordinates, one end coordinates, one height, one colours, and so on, and you with to create some conventional data-structure for reporting, with the schema below:

```
[
    { "start": w0, "end", x0, "height": y0, "colour": z0 },
    { "start": w1, "end", x1, "height": y1, "colour": z1 }
]
```

where `w, x, y, z` etc are EoEs. To do this, you use a `StructTemplate`.

A StructTemplate is a tree. You build leaf StructTemplates out of constants and EoEs and combine them to build your single template. You can then, for example, serialize this finished StructTemplate and you will get your nicely formatted JSON out. The data remains as compact as it is in the EoEs: nowhere is it expanded out into a very long string nor to giant data structure.

This is directly analagous to the way you would build such an object programatically in any other language, first out of your constants and variables, later built into arrays, objects etc, and when finished emit it. Though, EoEStruct has some special types to make things more useful in our context it is basically the same process.

### Constant StructTemplates

The simplest StructTemplate is a constant. This always emits that exact constant. You can work out what these do by yourself!

```
    pub fn new_number(input: f64) -> StructTemplate;
    pub fn new_string(input: String) -> StructTemplate;
    pub fn new_boolean(input: bool) -> StructTemplate;
    pub fn new_null() -> StructTemplate;
```

### Adding variables

Chances are your data-structure isn't a constant, so you'll need to introduce some variables, in the form of EoEs. To do this you need a `StructVarGroup`. You can just create these whenever you wish:

```
    pub fn new() -> StructVarGroup;
```

All the EoEs in a StructVarGroup are iterated through together (for example, you'd have a single group for your start and end co-ordinates, biotypes, etc).

Once you have a StructVarGroup, you can start putting each of your EoEs into it as `StructVar`s.

```
    pub fn new_number(group:&mut StructVarGroup, input: EachOrEvery<f64>) -> StructVar;
    pub fn new_string(group:&mut StructVarGroup, input: EachOrEvery<String>) -> StructVar;
    pub fn new_boolean(group:&mut StructVarGroup, input: EachOrEvery<bool>) -> StructVar;
```

### Iterating through groups of EoEs

Now you've created a `StructVarGroup` with some values in it, there will be a point in your template where you want one entry per element of the EoEs in that group. For example, in our motivating example of the array of objects, this will be the array at the very top level of the template (but need not be, in general).

```
    pub fn new_all(vars: &mut StructVarGroup, expr: StructTemplate) -> StructTemplate;
```

`expr` is the sub-template for each element.  If you like, this template node works a bit like a "for" loop.

At the moment we don't have any way of accessing the values of our EoE in that sub-template, which we certainly need for it to be useful! Do it like this:

```
    pub fn new_var(input: &StructVar) -> StructTemplate;
```

Of course, this template must be inside a relevant `new_all` for that variable.

### Simple Example

At least we're at the stage where we can illustrate a very simle realistic use case. Say you have a single EoE called `a` containing the values `[1,2,3,4,5]`. And you just want to serialise it as an array, just like that.

```
  let group = StructVarGroup::new();            // create group
  let var_a = StructVar::new_number(group,a);   // add a to group

  let element = StructTemplate::new_var(var_a); // each element of the array is just our value
  let template = StructTemplate::new_all(group,element); // create array by repeating element
```

### Adding objects (ie "maps", "dicts") to templates

You can add objects to a StructTemplate. Such objects are made out of pairs of keys and values. The key must be a constant string, but the value can be any StructTemplate. 

```
    pub fn new(key: &str, value: StructTemplate) -> StructPair;
```

An EoE is created of these pairs to make an object StructTemplate.

```
    pub fn new_object(input: EachOrEvery<StructPair>) -> StructTemplate;
```

(Though, for exciting corner-cases the argument is itself is an EoE, you'll almost always want an EachOrEvery::each([]) here, and so you can imagine that it simply takes a vector of pairs.

### More realistic example

We're now in a position to create a more realistic camples. Say you have two EoEs called `start` and `end`, of the same length and you want to put them into a structure of the form

```
  [ { "start": s0, "end": e0 }, { "start": s1, "end": e1 }, { "start": s2, "end": e2 }, ... ]
```

You could do it like this:

```
  let group = StructVarGroup::new();
  let var_start = StructVar::new_number(group,start);
  let var_end = StructVar::new_number(group,end);

  let element = StructTemplate::new_object(EachOrEvery::each([
      StructPair::new("start",StructTemplate::new_var(var_start)),
      StructPair::new("end",StructTemplate::new_var(var_end))
  ]));
  let template = StructTemplate::new_all(group,element);
```

### Adding fixed arrays

Sometimes you want to add arrays of known length, working as tuples, rather than objects. Note that this is fundamentally a different case from the more common `new_all` described above, where the array iterates through your EoEs. To continue the previous example, rather than the serialisation described there, you might want it in the form.

```
  [ { "range": [s0,e0] }, { "range": [s1,e1] }, { "range": [s2,e2] }, ... ]
```

For which `new_array` is provided and which you can do like this:

```
  let group = StructVarGroup::new();
  let var_start = StructVar::new_number(group,start);
  let var_end = StructVar::new_number(group,end);

  let range_tuple = StructTemplate::new_array(EachOrEvery::each([
      StructTemplate::new_var(var_start),
      StructTemplate::new_var(var_end)
  ]);
  let element = StructTemplate::new_object(EachOrEvery::each([
      StructPair::new("range",range_tuple)
  ]));
  let template = StructTemplate::new_all(group,element);
```

### Advanced: Conditions

Templates can be wrapped in a condition template, which also takes a variable (as a `StructVar`). As the group is iterated through, if the variable is truthy then the condition has no effect and the subtemplate is rendered; otherwise it is as if the subtemplate isn't there at all. Conditions can only go inside arrays and pair values (for objects), wherein a falsy value means that the element and pair are skipped (repsectively) as the array/object is built.

For example, it could be that you are building our standard "array of objects" structure and there's a key in the object called "protein" with an id value from the EoE `protein` which should only be present when another EoE, `protein_present`, is true, otherwise the key be missing entirely.

```
  let group = StructVarGroup::new();
  let var_protein = StructVar::new_number(group,protein);
  let var_protein_present = StructVar::new_number(group,protein_present);
  ...
  let element = StructTemplate::new_object(EachOrEvery::each([
      ...
      StructPair::new("protein",StructTemplate::condition(
          var_protein_present,
          StructTemplate::new_var(var_protein)
      )
  ]));
  let template = StructTemplate::new_all(group,element);
```

### Advanced: late binding

Sometimes you want to add an EoE *after* a template has been generated, it's just not known at the time of template generation. These are known as "late" bindings and are passed at serialisation time. A placeholder "late" variable is passed when the template is generated and then when it is serialised, a `LateValues` object is passed containing the values of any "late" variables.

Template generation takes a non-trivial amount of time, so it makes sense not to do it too often, but "late" variables are a little slower and more awkward than regular variables if there is no need to use them.

```
    pub fn new_late(group:&mut StructVarGroup) -> StructVar;
    pub fn new() -> LateValues { LateValues(HashMap::new()) }
    pub fn add(&mut self, var: &StructVar, val: &StructVar) -> StructResult; // in LateValues
```

### Actually serialising

The actual method to serialise is `eoestack_run` but you will usually use a convenience function for the serialisation format of your choice, for exmaple `struct_to_json`.

To use these you will need a `StructBuilt` not your `StructTemplate`. Use `StructTemplate::build()` to acheive create it.
