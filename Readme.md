# SLRConfig

[![Build Status](https://travis-ci.org/SiegeLord/SLRConfig.png)](https://travis-ci.org/SiegeLord/SLRConfig)

[Documentation](http://siegelord.github.io/SLRConfig/doc/slr_config/)

SLRConfig is a simple configuration format. It supports tables (mappings of
strings to elements) and arrays of elements, where an element may be a string,
an array or a table. Despite only supporting arrangements of strings, it's
permissive syntax and a standard implementation that preserves the location of
where each element was assigned allows the programmer to add support for any
other type with the same quality of error messages as if it were built in.

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
~~~

## Format description.

The language grammar is described using the [GOLD
meta-language](http://goldparser.org/doc/grammars/index.htm). Essentially,
character sets are specified using set notation, terminals are specified using
regular expressions and productions are specified using BNF.

## Lexical grammar

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

Only line comments are supported. The character `#` starts a line comment which
ends at a newline (LF). The comment can start anywhere on a line.

## Parser grammar and semantics

### Strings

~~~
<String> ::= NakedEscapedString
           | QuotedEscapedString
           | RawString0
           | RawString1
           | RawString2

<StringExpr> ::= <String>
               | <StringExpr> '~' <String>
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

#### String concatenation

Multiple strings can be contatenated using the `~` operator.

### Tables

~~~
<OptComma> ::= ','
             |
<Table> ::= '{' <TableContents> '}'

<TableElement> ::= <String> <Table>
                 | <String> '=' <Array>
                 | <String> '=' <StringExpr>

<TableElements> ::= <TableElement>
                  | <TableElements> <OptComma> <TableElement>

<TableContents> ::= <TableElements> <OptComma>
                  |
~~~

### Arrays

~~~
<Array> ::= '[' <ArrayContents> ']'

<ArrayElement> ::= <StringExpr>
                 | <Table>
                 | <Array>

<ArrayElements> ::= <ArrayElement>
                  | <ArrayElements> ',' <ArrayElement>

<ArrayContents> ::= <ArrayElements> <OptComma>
                  |
~~~
