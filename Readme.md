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
on = a, single = line
just another string = -1.5

table
{
	array = [a, b]
}
~~~

## Format description.

TODO
