# Earp

## Introduction

Earp is a domain-specific language for the genome-browser. It is interpreted within the web-rbrowser. The main design directions are:

* clear and easy for novice wuers
* efficient handling of large datasets
* fast to implement the interpreter
* small interpreter

## Type System

Earp has the following base types.

* booleans
* numbers
* bytes
* identifiers
* strings

While there are no sigils to demonstrate the contents of a variable, the complier ensures that which of these values a variable contains is always unambiguous. For example, assigning a variable an integer in one branch of a statement and a boolean in another, and then attempting to use it later is a compile-time error. Coercions are always explicit. So while earp looks like it plays fast and loose with types (no explicit typing and no sigils), it is actually rather strict.

Earp also has one-dimensional homogenous arrays of each of these types. There are no structured types beyond this: no higher-dimensional arrays and no enums/structs. This is a concession of expressiveness for performance. It is mitigated by:

 * a very liberal set of chrarcters allowed in variable names;
 * standard functions to emulate higher-dimensional arrays;
 * "out" arguments to functions, to allow multiple returns;
 * the bulk-argument syntax keeping function argument lists small;
 * a rich slicing system allowing fast array operations.

 The main benefit of this restriction is to allow almost all data traversals to take place in parallel (rather than with loops) which removes much of the interpreter overhead from processing large datasets. Large changes to the data can occur entirely in the CPU within various layers of cache and with effective branch prediction, making them incredibly fast for an interpreted language. This is important as our datasets are large and attempting to manipulate them in explicit loopsin an interpreted language would be deathly slow. A more sophisitcated example of this approach is numpy in python, where similar issues arose.

## Arrays

Arrays are generally manipulated using slices. Slice syntax uses the square-brackets more typically used for retrieving by index (so don't confuse it for that!). The syntax is `array[filter]` where `array` is an arbitrary array and `filter` must be a boolean array (or a boolean, see later). The boolean array selects indexes from the array, and produces a new sub-array. Importantly, the slice is by reference and modifiable, allowing filtered updates. Boolean arrays are implemented efficiently for very large, sparse arrays for just this reason.

```
values := [101,102,103,104];

filter := [false,true,false,true];
values[filter] += 100;

print(values);
// [101,202,103,204]
```

To allow filtering to be generally useful, most predicates across the language (such as equality tests, greater/less than operators, etc) allow one argument to be an array, evaluationg to a boolean array, rather than a boolean.

```
values := [101,102,103,104];

filter := values > 102; // [false,false,true,true]
values[filter] += 100;

print(values);
// [101,102,203,204]
```

Naturally, the filter expression can be moved inline. This allows mapping traversals. For example, the following is equivalent to the above.

```
values := [101,102,103,104];

values[values > 102] += 100;

print(values);
// [101,102,203,204]
```

The builtin funciton `index()` generates an appropriate boolean array to access just a single value, allowing single-instance access. (Don't worry about the array being too short for now, see the section on iterators and simply assume that it always has sufficient implicit trailing falses.

```
values := [101,102,103,104];

values[index(2)] += 100;

print(values);
// [101,102,203,104]

// Let's see how that works
print(index(2));
// [false,false,true]
```

The `range(offset,length)` builtin returns a boolean array, where from offset to offset+length, the boolean is true.

```
print(range(4,2));
// [false,false,false,false,true,true]
```

Slices are useful for updating mutliple arrays representing structured data.

```
// some data
gene.name    := ["A","B","C","D","E","F"];
gene.start   := ["11","22","33","44","55","66"];
gene.end     := ["17","27","37","47","57","67"];
gene.is_nice := ["y","y","n","n","y","y"];

// we only want nice genes
nice := gene.is_nice == "y";

gene.name  := gene.name[nice];
gene.start := gene.start[nice];
gene.end   := gene.end[nice];
```

The `copy()` buitin creates an independent copy of an array, as does passing the array to a function using _in_ mode. All copies are lazy, and therefore efficient if the variable is unchanged.

## Iterators

Functions and procdedures can be defined as accepting "iterators" for in and in/out arguments. Such functions accept both the array and non-array agruments. A non-array argument is equivalent to an infinite sequence. These values are rather opaque: there are very few operations available on iterators to the programmer, but many builtin functions accept them, including, importantly, the slice operator. Therefore, the slice operator also accepts a plain boolean to update all or none of the values.

Iterators are mainly used in functions which also take other arrays and allow subsequent arguments to co-vary with the main array or else take a fixed value. Often they also include a convention as to allow over-short or over-long array arguments as well. For example, the filter operator assumes trailing falses and ignores excess values.

Iterators are the only datatype polymorphism or ambiguity allowed in earp.

## (Explicit) Type Coercion

Type-coercion builtin functions allow inter-conversion, named after the target types: `to_number`, `to_string`, `to_boolean`, `to_bytes`, `to_number_array`, `to_string_array`, `to_boolean_array`, `to_bytes_array`. Arrays can alsso be down-converted using thsee functions.

| in / out | n | s | bo | by | na | sa | boa | bya |
|----------|---|---|----|----|----|----|-----|-----|
| n        | I | F | B  | V  | S  | -  | -   | -   |
| s        | P | I | P  | U  | -  | S  | -   | -   |
| bo       | B | F | I  | B  | -  | -  | S   | -   |
| by       | V | U | B  | I  | V+ | -  | -   | S   |
| na       | S | - | -  | V+ | I  | F+ | B+  | V+  |
| sa       | - | S | -  | -  | P+ | I  | P+  | U+  |
| boa      | - | - | S  | -  | B+ | P+ | I   | B+  |
| bya      | - | - | -  | S  | V+ | U+ | B+  | I   |

* `I` -- no change
* `F` -- formatted
* `P` -- parsed
* `U` -- utf8 conversion
* `B` -- 1 = true, 0 = false
* `V` -- unsigned byte value 0-255
* `S` -- single member / first member
* `+` -- on each

The scalar conversions accept all other scalar types. to_string() formats its input arguments. Number

* `to_number` accepts a string, a number, a boolean, and bytes.
* `to_string` accepts a string, a number, a boolean, and bytes.
* `to_boolean` accepts a string, a number, a boolean, and bytes.
* `to_bytes` accepts a string, a number, a boolean, bytes, and a number array.

## Conditionals and Loops

Conditionals and loops are available in earp but best avoided for performance. Loops are particularly heinous.

Conditionals accept a boolean.

```
if (test) {
    print("passed test");
} else {
    print("failed test");
}
```

Loops accept any array format.

```
loop name from (names) {
    print(name);
}
```

Parentheses are mandatory as they may contain an arbitrary expression.

## Literals, Construction, and Constants

* **numbers** numbers can be specified in standard number formats, including scientific notation (with `e`), hex (with `0x`) and octal (with `0`).
* **booleans** can be specified with `true` and `false`.
* **strings** can be specified with double quotes. Multiline strings can be spceified with triple quotes, `"""`. Leading whitespace upto the first newline or non-whitespace are ignored as is trailing whitespace before the closing quote and on the same line.
* **bytes** have no literal syntax and are best created using an array of numbers or a string.
* **identifiers** have no general construction method.

Arrays can be constructed with square brackets `[]`. Array values of the correct type are also allowed and interpolated. This allows merging of arrays, appending and so on.

```
x := [1,2,3]; // simple construction
x := [42,x]; // append 42 onto x
z := [x,y]; // merge arrays x and y (if both arrays)
```

Comments have C++-like syntax. Multiline comments use `/* */` and to end-of-line comments ue `//`.

## Functions and Procedures

Earp incluides functions and procedures. The difference between them is simply syntactic: a function has an out parameter to which it evaluates which allows it to be used in a n expression whereas a procedure must be a statement.

There are three modes of parameters to functions:

| name | sigil | description |
|------|-------|-------------|
_in_ | `<` (optional, default)| a call-by-value parameter (regular parameter |
| _in/out_ | `<>` | a call-by-reference parameter (value can be both read and written and is reflected in calling value) |
| _out_ | `>` | value propagates out of function, value is initially default | 

These sigils must be present (and match) both in the signature and call (except that `<` is always optional and usually best ommitted for clarity).

Functions and procedures are introduced with `function` and `procedure`. The `return` statement returns a value from the function (otherwiset the default is returned). It is a compile-time error for branches not to include an explicit return. Earp compilers are not required to detect inifinite loops, so this includes (unused) returns after such cases.

```
function add_two(input, >half_way) {
  halfway := input + 1;
  return halfway + 1;
}

x := 2;
z := add_two(x,>y);
print(x); // 2
print(y); // 3
print(z); // 4
```

Function and procedure implementations must be unique with respect to the type of their in and in/out parameters.

## Bulk Argument Sytnax

Bulk-agrument syntax allows all variables defined up to a period (`.`) to be passed as arguments. When used as an argument or parameter type, it is a suffixed `..`. Bulk-argument syntax cannot be used in return types for functions. Bulk-argument syntax can be used with all parameter types. For example, the gene filtering example above can be implemented in a utility procedure as follows.

```
procedure only_nice_genes(<>data..) {
  nice := data.is_nice == "y";

  data.name  := data.name[nice];
  data.start := data.start[nice];
  data.end   := data.end[nice];
}

// some data
gene.name    := ["A","B","C","D","E","F"];
gene.start   := ["11","22","33","44","55","66"];
gene.end     := ["17","27","37","47","57","67"];
gene.is_nice := ["y","y","n","n","y","y"];

// we only want nice genes
only_nice_genes(<>gene..);
```

or the same things, using iterators with a pluggable predicate

```
procedure filter_genes(<>data.., pred) {
  data.name  := data.name[pred];
  data.start := data.start[pred];
  data.end   := data.end[pred];
}

// some data
gene.name    := ["A","B","C","D","E","F"];
gene.start   := ["11","22","33","44","55","66"];
gene.end     := ["17","27","37","47","57","67"];
gene.is_nice := ["y","y","n","n","y","y"];

// we only want nice genes
filter_genes(<>gene.., gene.is_nice=="y");
```

## Imports

Declarations cannot be nested so there are thre kinds of scope: 
* within a single procedure/function;
* at the file level (exported);
* at the file level (not-exported).

When a file is included, the exported elements at the file level become visible as not-exported elements within the new file.

*Variables*: only variables within that function/procerdure's scope are visible within that function. Variables defined at the file level cannot be exported.

*Functions:* functions with the `export` keyword prefix are exported (and so made available in another file when imported). Other functions exist at the non-exported file level. No functions are defiend within a function scope. References within a function are to one of the file scopes. Names in the file scope must be orthogonal. There are no disambiguating procedures for multiply defined functions sobe careful with your names. It is an error for two functions to exist with the same name and visible within a scope.

Importing is achieved with the `import` keyword. It takes a string constant giving a path relative to the current file or a URL.

## Structured data

In a typical representation of structured data, each low-level data item contains a reference to a higher-level object (as an index). The builtin `pick` uses this reference to retrieve the property for the lower-level items.

Consider exons in genes. In many cases (such as drawing) we can consider exons as a flat array. In other cases they need to be linked to their gene. In this case it makes more sense to have a reference _from_ the exons into the gene array.

```
exon.start := [A,B,C,D,...];
exon.end   := [W,X,Y,Z,...];
exon.gene  := [m,m,m,n,...];

// create biotype array for exons (eg to place correctly)

exon.biotype := pick(exon.gene,gene.biotype);

green_exons := exon.biotype$$ == "protein_coding";
blue_exons := exon.biotype == "nmd";

special_exons := green_exons || blue_exons; // etc

exon.colour := repeat(len(exon.start), "red");
exon.colour[green_exons] := "green";
exon.colour[blue_exons] := "blue";
```

On the other hand, we can also update the lower level object based on properties of the high-level one.

```
// fred at index m, bob at n
gene.name := [..., ..., "fred", ..., "bob", ...]; 
exon.start := [A,B,C,D,...];
exon.end   := [W,X,Y,Z,...];
exon.gene  := [m,m,m,n,...];

focus_gene := gene.name == "fred";
focus_index := position(focus_gene);
focus_exon := exon.gene == focus_index;

exon.colour[focus_exon] := "purple";
```

Very complex structured types should be implemented with domain-specific identifiers with the relevant methods implemented in the interpreted rather than as ADTs. However id is important to keep biological modelling out of identifiers so such implementations which touch on bioligy should be as abstract and visual as possible.

## ABD convention

If is also possible to represent nested arrays wqith auxilliary arrays giving the offset an length of each sub-array. By convention these are named A and B, with the data array named D. for exmaple:

```
// [["A","B","C"],["D","E"],[]]
x.a := [0,3,5];
x.b := [3,2,0];
x.d := ["A","B","C","D","E"];
```

A number of utility builtins allow use of this format.

* `abd_pick(a,b)` returns a bool array selecting the lower-level elements at the given location.
* `abd_index(x,a,b)` returns the indexes in the given a and b arrays of all true values as a number array.

If used carefully, abd can be used for higher-dimensional arrays: rather than applying the filters to D arrays containing data, higher level arrays filter lower level A and B arrays.
