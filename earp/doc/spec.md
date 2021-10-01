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

Arrays are always passed by reference despite function argument mode, though it remains conventional to give the correct sigil (eg `<>`) for clarity. The `copy()` buitin creates an independent copy of an array.

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

## Literals

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

Bulk-agrument syntax allows all variables defined up to a perion (`.`) to be passed as arguments. The sigil is a suffixed `..`. Bulk-argument syntax can be used with all parameter types. For example, the gene filtering example above can be implemented in a utility procedure as follows.

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

## Multi-dimensional Arrays

Multi-dimensional arrays are represented using one-dimensional arrays plus two extra structuring variables -- offset and length -- for each extra dimension (conventionally named `.a` and `.b`) in addition to the actual data (conventionally named `.d`). Further levels index into the lower level offset and length arrays, and so on. This is efficient but awkward. Fortunately, multi-dimensional data is rare in our domain. The extra effort of this approach is generally paid-off with fast computation even with large datasets.

```
// Representation of [[1,2,3],[4],[5,6]]
x.a%% := [0,3,4];       // offsets
x.b%% := [3,1,2];       // lengths
x.d%% := [1,2,3,4,5,6]; // the data

// Representation of [[[1,2],[3,4]],[[5,6],[7,8]]]
x.0.a%% := [0,4];     // top-level offsets to the .1 arrays
x.0.b%% := [4,4];     // top-level lengths to the .1 arrays
x.1.a%% := [0,2,4,6]; // inner-level offsets to the .d array
x.1.b%% := [2,2,2,2]; // inner-level lengths to the .d array
x.d%%   := [1,2,3,4,5,6,7,8];
```

Note that outer dimensions refer into inner dimensions and all data can be sparse (typically after updates to the arrays, etc). `compactX(value..)` compacts higher-dimensional arrays for integer value `X` for small X. `compact(<>value%%,<>offset%%,<>length%%)` (and equivalent for other types) can be used for more complex cases. See appendix to example of compationg.

Slices are used to work with multi-dimensional arrays, including reading and non-shape-changing writes via the `multi_index()` and `multi_slice()` builtins.

See appendix for exmaples of sub-slicing multi-dimensional arrays. `multi_reverse()` yields the offsets of ranges overlapping a predicate's true values for reverse lookup as a boolean array.

When updating an array in a shape-changing way (replacing a row with another of a different size, say), the memory freed is not reclaimed except after a subsequent compation. In most circumstances, it makes sense for the memory simply to "fall-off-the-end-of-the-program" unstead. The new data is simply appended to the `.d` array (and any necessary lower-level `.a` and `.b` arrays). (See appendix for examples).

# Appendices

These monstroseties are provided as a cookbook. Needless to say they are best avoided or, if necessary, factored into well-documented functions.

## Appendix: compacting multi-dimensional arrays

```
// A non-compact representation for [[1,2],[3,4]]
x.a%% := [2,5];
x.b%% := [2,2];
x.d%% := [0,0,1,2,0,3,4];
compact2(x..);
print(x.a%%); // [0,2]
print(x.b%%); // [2,2]
print(x.d%%); // [1,2,3,4]
```

Doing it long-hand.

```
// A non-compact representation for [[[1,2],[3,4]],[[5]],[[6]]]
x.0.a%% := [2,7];
x.0.b%% := [2,2];
x.1.a%% := [0,0,1,5,0,0,0,8,9];
x.1.b%% := [0,0,2,2,0,0,0,1,1];
x.d%%   := [0,1,2,0,0,3,4,0,5,6];

// compact3() would do the job for us here, but let's do it in bits

// compact the .1. variagles
compact(x.1.a%%,x.0.a%%,x.0.b%%);
compact(x.1.b%%,x.0.a%%,x.0.b%%);

// compact .d
compact(x.d%%,x.1.a%%,a.1.b%%);
```

## Appendix: Sub-Slicing Multi-Dimensional Arrays

```
// Assume data%% contains some rep of [[1,2,3],[4],[5,6]]
third_row?? := multi_index(data.0.a%%,data.0.b%%,2);
print(data.d%%[third_row??]) // [5,6]

// Flattening of second and third rows. Useful for updates.
rows?? := [false,true,true];
rows_23?? := multi_slice(data.0.a%%,data.0.b%%,rows??);
print(data.d%%[rows_23??]); // [4,5,6]

// Flattening of rows with fewer then three members
rows?? := x.0.b%% < 3;
short_rows?? := multi_slice(data.0.a%%,data.0.b%%,rows??);
print(data.d%%[short_rows??]); // [4,5,6]

// New array, y, containing only short rows
rows?? := x.0.b%% < 3;
y.0.a%% := x.0.a%%[rows??];
y.0.b%% := x.0.b%%[rows??];
y.0.d%% := x.0.d%%;

// Assume x contains [[[1,2],[3]],[[4],[5,6]]]
// create y containing just second row, ie [[4],[5,6]]
row2_a% := scalar(x.0.a%%[index(1)]);
row2_b% := scalar(x.0.b%%[index(1)]);
second_row?? := range(row_2_a%,row_2_b%);
y.0.a%% := x.1.a%%[second_row??];
y.0.b%% := x.1.b%%[second_row??];
y.d%%   := x.d%%;

// create y containing only rows containing any element over 5.
over5?? := x.d%% > 5; // [f,f,f,f,f,t]
match.1?? := multi_reverse(over5??,x.1.a%%,x.1.b%%); // [f,f,f,t]
match.0?? := multi_reverse(match.1??,x.0.a%%,a.0.b%%); // [f,t]
y.0.a%% := x.0.a%%[match.0??];
y.0.b%% := x.0.b%%[match.0??];
y.1.a%% := x.1.a%%[match.1??];
y.1.b%% := x.1.b%%[match.1??];
```

## Appendix: replacing array part (reshaping)

```
// Assume x contains [[[A,B],[C]],[[D],[E,F]]]
// we want to append [[E],[F,G,H]]
new.a%% := [0,1];
new.b%% := [1,3];
new.d$$ := ["E","F","G","H"];

//
len_new% := len(new_a%%);
len_1% := len(x.1%%);
len_d% := len(x.d$$);

// add the actual data
append(<>x.d$$,new.d$$);

// at level 1 we tell it about the new data in d
append(<>x.1.a%%,new.a%% + len_d%)
append(<>x.1.b%%,new.b%%);

// at level 0 we tell it about the new rows at level 1
append(<>x.0.a%%,len_1%);
append(<>x.0.b%%,len_new%);
```

XXX replace, delete row