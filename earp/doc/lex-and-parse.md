# Lexing

## Summary

* Tokenising
* Remove comments and whitespace
* numbers
* string constants
* multi-line strings constants
* multi-character operators  

# Parsing

## Grammar

```
constant := `false` | `true` | number | string | mutli_line_string;

sigil := `<` | `>` | `<>`;
argument := sigil? (( identifier `..`) | expression);
call := identifier `(` argument (`,` argument)* `)`;

definition := `export`* (`function`|`procedure`) identifier `(` argument* `)` `{  function_statement* `}`

constructor := `[` expression (`,` expression)* `]`;
expr0 := constant | identifier | call | constructor | `(` expression `)`;
expr1 := expr0 (`[` expression `]`)?;
expr2 :=  expr1 | (`+``-``!`) expr2;
expr3 := expr2 | expr3 ([`*``/``%`]) expr2;
expr4 := expr3 | expr4 ([`+``-`]) expr3;
expr5 := expr4 | expr5 ([`>``>=``<=``<`]) expr4;
expr6 := expr5 | expr6 `==` expr5;
expression := expr6;

left_expression := identifier (`[` expression `]`)?;

statement :=
  left_expression [`+=``-=``*=``/=``||=``&&=``:=`] expression |
  call;

function_statement :=
  statement |
  `return` expression |
  `if` expression `{` function_statement* `}` |
  `loop` identifier `from` `(` expression `)` `{` function_statement* `}`;

toplevel_statement :=
  statement |
  definition |
  `import` string |
  `if` expression `{` statement* `}` |
  `loop` identifier `from` `(` expression `)` `{` statement* `}` |
  file-change;

```

## Parse Tree

Parsing replaces operators with equivalent builtin calls.