# Earp

## Introduction

Earp is a domain-specific language for the genome-browser. It is interpreted within the web-rbrowser. The main design directions are:

* clear and easy for novice wuers
* efficient handling of large datasets
* fast to implement the interpreter
* small interpreter

## Type System

Earp has the following base types.

* booleans (`?` sigil)
* numbers (`%` sigil)
* bytes (`!` sigil)
* identifiers (`{type}` sigil)
* strings (`$` sigil)

Each has an identified default value, to which variables are assigned before they are updated.

Variables are namespaced according to disjoint types and use a sigil prefix (eg `value%`) (therefore there is no implicit coercion). Sigils are suffixed rather than prefixed to make the identifier sigil natural, it is also consistent with Hungarian notation so isn't unprecidented.

Earp also has one-dimensional homogenous arrays of each of these types. These are indicated by brackets after the sigil (for example `my_var??`). Identifiers just double the braces `{{kind}}` Sigils help with informative error reporting.

There are no structured types beyond this, no higher-dimensional arrays and no enums/structs. This concession of expressiveness for performance is mitigated by:

 * very liberal set of chrarcters allowed in variable names;
 * standard functions to emulate higher-dimensional arrays;
 * out parameters to functions to allow multiple returns;
 * bulk-argument syntax;
 * a rich slicing system.

 The main benefit of this restriction is to allow almost all data traversals to take place in parallel (rather than with loops) which removes much of the interpreter overhead from processing large datasets.

## Arrays

Arrays are generally manipulated using slices. Slice syntax uses the square-brackets more typically used for retrieving by index (so don't confuse it for that!). The syntax is `array%%[filter??]` where `array%%` is an arbitrary array (here of numbers) an `filter??` must be a boolean array (or a boolean, see later). The boolean array selects indexes from the array and produces a new sub-array. Importantly, the slice is by reference, allowing filtered updates. Boolean arrays are implemented efficiently for very large, sparse arrays.

```
values%% := [101,102,103,104];
filter?? := [false,true,false,true];
values[filter??] += 100;
print(values%%);
// [101,202,103,204]
```

To facilitate filtering, most predicates allow one argument to be an array, generating a boolean array rather than a boolean.

```
values%% := [101,102,103,104];
filter?? := values%% > 102;
values[filter??] += 100;
print(values%%);
// [101,102,203,204]
```

Naturally, the filter expression can be moved inline. This allows mapping traversals. For example, the following is equivalent to the above.

```
values%% := [101,102,103,104];
values[values%% > 102] += 100;
print(values%%);
// [101,102,203,204]
```

Indexes after the last one present in the boolean array are assumed to be false. The builtin funciton `index()` generates an appropriate boolean array to access just a single value, allowing single-instance access.

```
values%% := [101,102,103,104];
values[index(2)] += 100;
print(values%%);
// [101,102,203,104]

// Let's see how that works
print(index(2));
// [false,false,true]
```

The `range(offset%,length%) -> ??` builtin returns a boolean where from offset to offset+length, the boolean is true.

```
print(range(4,2));
// [false,false,false,false,true,true]
```

Slices are useful for updating mutliple arrays representing structured data.

```
// some data
gene.name$$    := ["A","B","C","D","E","F"];
gene.start$$   := ["11","22","33","44","55","66"];
gene.end$$     := ["17","27","37","47","57","67"];
gene.is_nice$$ := ["y","y","n","n","y","y"];

// we only want nice genes
nice?? := gene.is_nice$$ == "y";

gene.name$$  := gene.name$$[nice??];
gene.start$$ := gene.start$$[nice??];
gene.end$$   := gene.end$$[nice??];
```

The `copy()` buitin creates an independent copy of an array, as does passing the array to a function using _in_ mode. All copies are lazy, and therefore efficient if the variable is unchanged.

## Iterators

Functions and procdedures can be defined as accepting "iterators" for in and in/out arguments. The sigil for an iterator, for use in the signature, is `&X`, where `X` is a base sigil. Such functions accept both the array and non-array agruments. A non-array argument is equivalent to an infinite sequence. These values are rather opaque: there are very few operations available on iterators to the programmer, but many builtin functions accept them, including the slice operator. Therefore, the slice operator also accepts a plain boolean to update all or none of the values. They are mainly used in functions which also take other arrays and allow subsequent arguments to co-vary with the main array or else take a fixed value.

## (Explicit) Type Coercion

Type-coercion builtin functions allow inter-conversion, named after the target types: `to_number`, `to_string`, `to_boolean`, `to_bytes`. Arrays can be down-converted using the `scalar` function.

## Conditionals and Loops

Conditionals and loops are available in earp but best avoided for performance. Loops are particularly heinous.

Conditionals accept a boolean.

```
if (test?) {
    print("passed test");
} else {
    print("failed test");
}
```

Loops accept any array format.

```
loop name$ from (names$$) {
    print(name$);
}
```

Parentheses are mandatory as they may contain an arbitrary expression.

## Literals and Construction

XXX


## Functions and Procedures

Earp incluides functions and procedures. The difference between them is syntactic. A function has an out parameter to which it evaluates which allows it to be used in a n expression whereas a procedure must be a statement.

There are three modes of parameters to functions:

| name | sigil | description |
|------|-------|-------------|
_in_ | `<` (optional, default)| a call-by-value parameter (regular parameter |
| _in/out_ | `<>` | a call-by-reference parameter (value can be both read and written and is reflected in calling value) |
| _out_ | `>` | value propagates out of function, value is initially default | 

Sigils must be present (and match) both in the signature and call.

Functions and procedures are introduced with `function` and `procedure`. The `return` statement returns a value from the function (otherwiset the default is returned). Function return type is identified with `-> <sigil>`. For example:

```
function add_two(input%, >half_way%) -> % {
  halfway% := input% + 1;
  return halfway% + 1;
}
```

Function and procedure implementations must be unique with respect to the type of their in and in/out parameters.

Bulk-agrument syntax allows all variables defined up to a perion (`.`) to be passed as arguments. The sigil is a suffixed `..`. Bulk-argument syntax cannot be used in return types for functions. Bulk-argument syntax can be used with all parameter types. For example, the gene filtering example above can be implemented in a utility procedure as follows.

```
procedure only_nice_genes(<>data..) {
  nice?? := data.is_nice$$ == "y";

  data.name$$  := data.name$$  [nice??];
  data.start$$ := data.start$$ [nice??];
  data.end$$   := data.end$$   [nice??];
}

// some data
gene.name$$    := ["A","B","C","D","E","F"];
gene.start$$   := ["11","22","33","44","55","66"];
gene.end$$     := ["17","27","37","47","57","67"];
gene.is_nice$$ := ["y","y","n","n","y","y"];

// we only want nice genes
only_nice_genes(<>gene..);
```

or the same things, using iterators with a pluggable predicate

```
procedure filter_genes(<>data.., pred&?) {
  data.name$$  := data.name$$  [pred&?];
  data.start$$ := data.start$$ [pred&?];
  data.end$$   := data.end$$   [pred&?];
}

// some data
gene.name$$    := ["A","B","C","D","E","F"];
gene.start$$   := ["11","22","33","44","55","66"];
gene.end$$     := ["17","27","37","47","57","67"];
gene.is_nice$$ := ["y","y","n","n","y","y"];

// we only want nice genes
filter_genes(<>gene.., gene.is_nice$$=="y");
```

## Imports

XXX

## Structured data

In a typical representation of structured data, each low-level data item contains a reference to a higher-level object (as an index). The builtin `pick` uses this reference to retrieve the property for the lower-level items.

Consider exons in genes. In many cases (such as drawing) we can consider exons as a flat array. In other cases they need to be linked to their gene. In this case it makes more sense to have a reference _from_ the exons into the gene array.

```
exon.start%% := [A,B,C,D,...];
exon.end%%   := [W,X,Y,Z,...];
exon.gene%%  := [m,m,m,n,...];

// create biotype array for exons (eg to place correctly)

exon.biotype$$ := pick(exon.gene%%,gene.biotype$$);

green_exons?? := exon.biotype$$ == "protein_coding";
blue_exons?? := exon.biotype$$ == "nmd";

special_exons?? := green_exons?? || blue_exons??; // etc

exon.colour$$ := repeat(len(exon.start%%), "red");
exon.colour$$[green_exons??] := "green";
exon.colour$$[blue_exons??] := "blue";
```

On the other hand, we can also update the lower level object based on properties of the high-level one.

```
// fred at index m, bob at n
gene.name$$ := [..., ..., "fred", ..., "bob", ...]; 
exon.start%% := [A,B,C,D,...];
exon.end%%   := [W,X,Y,Z,...];
exon.gene%%  := [m,m,m,n,...];

focus_gene?? := gene.name$$ == "fred";
focus_index%% := position(focus_gene??);
focus_exon?? := exon.gene%% == focus_index%%;

exon.colour$$[focus_exon??] := "purple";
```

Very complex structured types should be implemented with domain-specific identifiers with the relevant methods rather than as ADTs. However id is important to keep biological modelling out of identifiers so such implementations which touch on bioligy should be as abstract and visual as possible.
