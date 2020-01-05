# SLRConfig

[![Build Status](https://travis-ci.org/SiegeLord/SLRConfig.png)](https://travis-ci.org/SiegeLord/SLRConfig)
[![](http://meritbadge.herokuapp.com/slr_config)](https://crates.io/crates/slr_config)

SLRConfig is a simple configuration format. It supports tables (mappings of
strings to elements) and arrays of elements, where an element may be a string,
an array or a table. Despite only supporting arrangements of strings, it's
permissive syntax and a standard implementation that preserves the location of
where each element was assigned allows the programmer to add support for any
other type with the same quality of error messages as if it were built in.

## Documentation

[Rust interface documentation](http://siegelord.github.io/SLRConfig/doc/slr_config/)

## Packages

* [slr_config](https://crates.io/crates/slr_config) - Convenient Rust interface.
* [slr_parser](https://crates.io/crates/slr_parser) - Parser.

## Example

Here's a sample snippet. The details of the syntax are explained further below.

~~~
# A comment.
key = value
statement = there's no need to quote the vast majority of characters
"sometimes
you" = "need
to"
"you can always escape ☺
" = you can always escape \u263a\n
on = a, single = line
just another string = -1.5
raw string = {{"embedded quote -> " <-"}}

table
{
	array = [a, b]
}

tagged_table = tag
{
   tagged_array = tag [1, 2]
}
~~~

## Format description.

The language grammar is described using the [GOLD
meta-language](http://goldparser.org/doc/grammars/index.htm). Essentially,
character sets are specified using set notation, terminals are specified using
regular expressions and productions are specified using BNF.

The root element is defined by the `<TableElements>` production.

## Lexical grammar

### Encoding

The string representation of the format is encoded using UTF-8.

### Character sets

~~~
{Raw String Chars} = {Printable} + {Whitespace}
{Escaped String Chars} = {Printable} + {Whitespace} - ["]

{String Middle} = {Printable} - {Whitespace} - [#='['']'{}$",~] + [' ']
{String Border} = {String Middle} - [' ']
~~~

### Terminals

~~~
NakedEscapedString = ({String Border} | '\' {Raw String Chars})
                     ({String Middle} | '\' {Raw String Chars})*
                     ({String Border} | '\' {Raw String Chars})
                   | ({String Border} | '\' {Raw String Chars})
QuotedEscapedString = '"' {Escaped String Chars}* '"'

RawString0 = '{{"' {Raw String Chars}* '"}}'
RawString1 = '{{{"' {Raw String Chars}* '"}}}'
RawString2 = '{{{{"' {Raw String Chars}* '"}}}}'
~~~

### Comments

Line comments are introduced using the `#`  character. Line comments can start
anywhere on a line and end at a newline (LF).

## Parser grammar and semantics

### Strings

~~~
<String> ::= NakedEscapedString
           | QuotedEscapedString
           | RawString0
           | RawString1
           | RawString2
~~~

Strings are used as keys in tables as well as one type of element that can be
in tables and arrays. There are two types of strings, escaped and raw.

#### Escaped strings

Escaped strings can have character escapes which are resolved during parsing.
The supported escapes are as follows:

- `\n` - Line feed
- `\r` - Carriage return
- `\t` - Horizontal tab
- `\0` - NUL
- `\\` - Backslash
- `\uxxxx` - Unicode character xxxx, where x's are lower-case hexadecimal digits
- `\Uxxxxxxxx` - Unicode character xxxxxxxx, where x's are lower-case hexadecimal digits

Invalid escapes are replaced by `�` (U+fffd) character. There are two types of
escaped strings, naked and quoted. Naked strings do not need quotes around
them, but are restricted by what characters they may contain.

#### Raw strings

Raw strings contain exactly the characters that are inside of them. The number
of leading braces must match the number of trailing braces, but otherwise can
be increased in case there's a quote character followed run of trailing braces
inside the string itself.

### Expressions

~~~
<Expansion> ::= '$' <String>

<Expr> ::= <String>
        |  <Expansion>
        |  <Expr> '~' <String>
        |  <Expr> '~' <Expansion>
~~~

Table and array elements are assigned the results of expressions. An expression
can result in a value (a string), a table or an array. Multiple strings can be
contatenated using the `~` operator.

Using the expansion syntax it is possible to fetch the value of a previously
assigned element. The elements are looked up by key from tables and by index
(zero based) from arrays. The lookup is performed by looking up the element in
the table/array that the element being assigned to is. If nothing was found,
then a higher level table/array is examined, and so on. The element being
looked up must be lexically prior in the source string to the element being
assigned to. The element being assigned to is never considered as an expansion
element.

### Tables

~~~
<OptComma> ::= ','
            |
<Table> ::= '{' <TableContents> '}'

<TableElement> ::= <String> <Table>
                |  <String> '=' <Array>
                |  <String> '=' <Expr>
                |  <String> '=' <TaggedTable>
                |  <String> '=' <TaggedArray>

<TableElements> ::= <TableElement>
                 |  <TableElements> <OptComma> <TableElement>

<TableContents> ::= <TableElements> <OptComma>
                 |
~~~

### Arrays

~~~
<Array> ::= '[' <ArrayContents> ']'

<ArrayElement> ::= <Expr>
                |  <Table>
                |  <Array>
                |  <TaggedTable>
                |  <TaggedArray>

<ArrayElements> ::= <ArrayElement>
                 |  <ArrayElements> ',' <ArrayElement>

<ArrayContents> ::= <ArrayElements> <OptComma>
                 |
~~~

### Tagged Tables

~~~
<TaggedTable> ::= <String> <Table>
~~~

Tagged tables are tables with a string tag attached to them. These are
surprisingly useful for representing named types.

### Tagged Array

~~~
<TaggedArray> ::= <String> <Array>
~~~

Tagged arrays are arrays with a string tag attached to them. These are
surprisingly useful for representing named types.

## Serde integration

Here is how various Rust constructs are encoded in SLDConfig.

### Numeric types

These are encoded using their string representation as naked strings.

### Boolean

`true` is encoded as `true` and `false` is encoded as `false`.

### Strings and characters

Strings are encoded as escaped strings, and are automatically escaped.

### Byte slices

Bytes are encoded as an array of integers.

### Slices and tuples

These are encoded as arrays.

### Maps

These are encoded as arrays of 2-element arrays (key/value pairs). These are
not encoded as tables because tables must have strings as their values, while
Rust mappings don't need to be. In the future, we might relax this.

### Option

`Some` is encoded as the contained value and `None` is encoded as an empty string.

### Units

Units are encoded as an empty string.

### Unit structs and variants

These are encoded by their names as strings.

### Newtype structs and variants

These are encoded by a 1-element tagged arrays. The tag is the name of the
struct/variant, and the array element is the value.

### Tuple structs and variants

These are encoded by a tagged arrays. The tag is the name of the
struct/variant, and the array elements are the tuple elements. When
deserializing tuple structs, a non-tagged array is also accepted.

### Structs and struct variants

These are encoded by a tagged tables. The tag is the name of the
struct/variant, and the table elements are the struct elements. When
deserializing tuple structs, a non-tagged table is also accepted.
