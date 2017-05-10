var searchIndex = {};

searchIndex["slr_config"] = {"doc":"This crate implements the parsing for the SLRConfig format. Basic usage revolves around the creation and use of the `ConfigElement` type, like so:","items":[[3,"Error","slr_config","The error type used throughout this crate.",null,null],[12,"kind","","",0,null],[12,"text","","",0,null],[4,"ErrorKind","","An enum describing the kind of the error, to allow treating different errors differenly.",null,null],[13,"ParseFailure","","A parse error has occured. This error is not recoverable.",1,null],[13,"InvalidRepr","","An object could not be parsed from its ConfigElement representation. This error is recoverable, but the value the the object is in an unspecified state.",1,null],[13,"UnknownField","","While parsing a struct from a table, an unknown field was found. This error is recoverable, and the struct is unaffected.",1,null],[13,"Custom","","A custom error available to 3rd party implementors. The semantics are defined by the 3rd party.",1,null],[3,"Source","","Annotated representation of the configuration source string.",null,null],[3,"ConfigElement","","A configuration element.",null,null],[4,"ConfigElementKind","","The kind of the configuration element.",null,null],[13,"Value","","A simple value, containing a string.",2,null],[13,"Table","","A table, which is a mapping of strings to configuration elements.",2,null],[13,"Array","","An array of configuration elements.",2,null],[11,"clone","","",3,{"inputs":[{"name":"self"}],"output":{"name":"configelement"}}],[11,"clone","","",2,{"inputs":[{"name":"self"}],"output":{"name":"configelementkind"}}],[11,"new_table","","Creates a new empty table.",3,{"inputs":[],"output":{"name":"configelement"}}],[11,"new_value","","Creates a new value.",3,{"inputs":[{"name":"t"}],"output":{"name":"configelement"}}],[11,"new_array","","Creates a new array.",3,{"inputs":[],"output":{"name":"configelement"}}],[11,"from_source","","Parses a source and returns a table. The source will be reset by this operation, and must not be used with any spans created from a previous parsing done with that source.",3,{"inputs":[{"name":"source"}],"output":{"name":"result"}}],[11,"from_str","","Parses a source and returns a table.",3,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"from_source_with_init","","Updates the elements in this table with new values parsed from source. If an error occurs, the contents of this table are undefined. The source will be reset by this operation, and must not be used with any spans created from a previous lexing done with that source.",3,{"inputs":[{"name":"self"},{"name":"source"}],"output":{"name":"result"}}],[11,"from_str_with_init","","Updates the elements in this table with new values parsed from source. If an error occurs, the contents of this table are undefined.",3,{"inputs":[{"name":"self"},{"name":"str"}],"output":{"name":"result"}}],[11,"kind","","Returns the kind of this element.",3,{"inputs":[{"name":"self"}],"output":{"name":"configelementkind"}}],[11,"kind_mut","","Returns the kind of this element.",3,{"inputs":[{"name":"self"}],"output":{"name":"configelementkind"}}],[11,"span","","Returns the span associated with this element.",3,{"inputs":[{"name":"self"}],"output":{"name":"span"}}],[11,"as_table","","If this is a table, returns a pointer to its contents.",3,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"as_table_mut","","If this is a table, returns a pointer to its contents.",3,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"as_value","","If this is a value, returns a pointer to its contents.",3,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"as_value_mut","","If this is a value, returns a pointer to its contents.",3,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"as_array","","If this is an array, returns a pointer to its contents.",3,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"as_array_mut","","If this is an array, returns a pointer to its contents.",3,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"insert","","Insert an element into a table or an array. Panics if self is a value. `name` is ignored if self is an array.",3,{"inputs":[{"name":"self"},{"name":"t"},{"name":"configelement"}],"output":null}],[11,"print","","Outputs the string representation of this element into into a printer.",3,{"inputs":[{"name":"self"},{"name":"option"},{"name":"bool"},{"name":"printer"}],"output":{"name":"result"}}],[11,"fmt","","",3,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[8,"ElementRepr","","Describes a way to convert a type to a ConfigElement and back.",null,null],[10,"from_element","","Updates the contents of `self` based on values in the element.",4,{"inputs":[{"name":"self"},{"name":"configelement"},{"name":"option"}],"output":{"name":"result"}}],[10,"to_element","","Creates an element that represents the contents of `self`.",4,{"inputs":[{"name":"self"}],"output":{"name":"configelement"}}],[14,"slr_def","","A macro to define the compile-time schemas for configuration elements. You can use this macro to define structs and enums, like so:",null,null],[11,"new","","",0,{"inputs":[{"name":"errorkind"},{"name":"string"}],"output":{"name":"error"}}],[11,"from_span","","Creates an error from a certain span of the source. The source argument, if set, must be set to the source that was used when the span was created.",0,{"inputs":[{"name":"span"},{"name":"option"},{"name":"errorkind"},{"name":"str"}],"output":{"name":"error"}}],[11,"clone","","",5,{"inputs":[{"name":"self"}],"output":{"name":"source"}}],[11,"clone","","",1,{"inputs":[{"name":"self"}],"output":{"name":"errorkind"}}],[11,"clone","","",0,{"inputs":[{"name":"self"}],"output":{"name":"error"}}],[11,"get_error","","",0,{"inputs":[{"name":"self"}],"output":{"name":"error"}}],[11,"next","","",5,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"eq","","",1,{"inputs":[{"name":"self"},{"name":"errorkind"}],"output":{"name":"bool"}}],[11,"ne","","",1,{"inputs":[{"name":"self"},{"name":"errorkind"}],"output":{"name":"bool"}}],[11,"fmt","","",1,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"new","","",5,{"inputs":[{"name":"path"},{"name":"str"}],"output":{"name":"source"}}]],"paths":[[3,"Error"],[4,"ErrorKind"],[4,"ConfigElementKind"],[3,"ConfigElement"],[8,"ElementRepr"],[3,"Source"]]};
searchIndex["slr_parser"] = {"doc":"","items":[[3,"Span","slr_parser","Type representing a certain sub-section of the source.",null,null],[3,"Token","","",null,null],[12,"kind","","",0,null],[12,"span","","",0,null],[3,"Source","","Annotated representation of the configuration source string.",null,null],[3,"Lexer","","A type handling the lexing.",null,null],[12,"cur_token","","",1,null],[12,"next_token","","",1,null],[3,"Error","","The error type used throughout this crate.",null,null],[12,"kind","","",2,null],[12,"text","","",2,null],[3,"ConfigString","","",null,null],[12,"kind","","",3,null],[12,"span","","",3,null],[3,"Printer","","A utility type for printing a configuration element.",null,null],[4,"StringQuoteType","","",null,null],[13,"Naked","","",4,null],[13,"Quoted","","",4,null],[4,"TokenKind","","",null,null],[13,"EscapedString","","",5,null],[13,"RawString","","",5,null],[13,"Assign","","",5,null],[13,"LeftBracket","","",5,null],[13,"RightBracket","","",5,null],[13,"LeftBrace","","",5,null],[13,"RightBrace","","",5,null],[13,"Dollar","","",5,null],[13,"Comma","","",5,null],[13,"Tilde","","",5,null],[13,"Eof","","",5,null],[4,"ErrorKind","","An enum describing the kind of the error, to allow treating different errors differenly.",null,null],[13,"ParseFailure","","A parse error has occured. This error is not recoverable.",6,null],[13,"InvalidRepr","","An object could not be parsed from its ConfigElement representation. This error is recoverable, but the value the the object is in an unspecified state.",6,null],[13,"UnknownField","","While parsing a struct from a table, an unknown field was found. This error is recoverable, and the struct is unaffected.",6,null],[13,"Custom","","A custom error available to 3rd party implementors. The semantics are defined by the 3rd party.",6,null],[4,"StringKind","","",null,null],[13,"EscapedString","","",7,null],[13,"RawString","","",7,null],[5,"get_string_quote_type","","",null,{"inputs":[{"name":"str"}],"output":{"name":"stringquotetype"}}],[5,"parse_source","","",null,{"inputs":[{"name":"source"},{"name":"v"}],"output":{"name":"result"}}],[11,"fmt","","",8,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",8,{"inputs":[{"name":"self"}],"output":{"name":"span"}}],[11,"new","","",8,{"inputs":[],"output":{"name":"span"}}],[11,"is_valid","","",8,{"inputs":[{"name":"self"}],"output":{"name":"bool"}}],[11,"combine","","",8,{"inputs":[{"name":"self"},{"name":"span"}],"output":null}],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",0,{"inputs":[{"name":"self"}],"output":{"name":"token"}}],[11,"eq","","",5,{"inputs":[{"name":"self"},{"name":"tokenkind"}],"output":{"name":"bool"}}],[11,"ne","","",5,{"inputs":[{"name":"self"},{"name":"tokenkind"}],"output":{"name":"bool"}}],[11,"fmt","","",5,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",5,{"inputs":[{"name":"self"}],"output":{"name":"tokenkind"}}],[11,"is_string","","",5,{"inputs":[{"name":"self"}],"output":{"name":"bool"}}],[11,"clone","","",9,{"inputs":[{"name":"self"}],"output":{"name":"source"}}],[11,"new","","",9,{"inputs":[{"name":"path"},{"name":"str"}],"output":{"name":"source"}}],[11,"next","","",9,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"fmt","","",6,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",6,{"inputs":[{"name":"self"}],"output":{"name":"errorkind"}}],[11,"eq","","",6,{"inputs":[{"name":"self"},{"name":"errorkind"}],"output":{"name":"bool"}}],[11,"ne","","",6,{"inputs":[{"name":"self"},{"name":"errorkind"}],"output":{"name":"bool"}}],[11,"fmt","","",2,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",2,{"inputs":[{"name":"self"}],"output":{"name":"error"}}],[11,"new","","",2,{"inputs":[{"name":"errorkind"},{"name":"string"}],"output":{"name":"error"}}],[11,"from_span","","Creates an error from a certain span of the source. The source argument, if set, must be set to the source that was used when the span was created.",2,{"inputs":[{"name":"span"},{"name":"option"},{"name":"errorkind"},{"name":"str"}],"output":{"name":"error"}}],[11,"new","","Creates a new lexer from a source. The source will be reset by this operation, and must not be used with any spans created from a previous lexing done with that source.",1,{"inputs":[{"name":"source"}],"output":{"name":"lexer"}}],[11,"get_source","","",1,{"inputs":[{"name":"self"}],"output":{"name":"source"}}],[11,"next","","",1,{"inputs":[{"name":"self"}],"output":{"name":"option"}}],[11,"clone","","",3,{"inputs":[{"name":"self"}],"output":{"name":"configstring"}}],[11,"fmt","","",3,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",7,{"inputs":[{"name":"self"}],"output":{"name":"stringkind"}}],[11,"fmt","","",7,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"eq","","",7,{"inputs":[{"name":"self"},{"name":"stringkind"}],"output":{"name":"bool"}}],[11,"ne","","",7,{"inputs":[{"name":"self"},{"name":"stringkind"}],"output":{"name":"bool"}}],[11,"append_to_string","","",3,{"inputs":[{"name":"self"},{"name":"string"}],"output":null}],[11,"to_string","","",3,{"inputs":[{"name":"self"}],"output":{"name":"string"}}],[11,"get_error","","",2,{"inputs":[{"name":"self"}],"output":{"name":"error"}}],[11,"new","","",10,{"inputs":[{"name":"w"}],"output":{"name":"printer"}}],[11,"value","","",10,{"inputs":[{"name":"self"},{"name":"option"},{"name":"str"}],"output":{"name":"result"}}],[11,"start_array","","",10,{"inputs":[{"name":"self"},{"name":"option"},{"name":"bool"}],"output":{"name":"result"}}],[11,"end_array","","",10,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[11,"start_table","","",10,{"inputs":[{"name":"self"},{"name":"option"},{"name":"bool"},{"name":"bool"}],"output":{"name":"result"}}],[11,"end_table","","",10,{"inputs":[{"name":"self"},{"name":"bool"}],"output":{"name":"result"}}],[8,"GetError","","",null,null],[10,"get_error","","",11,{"inputs":[{"name":"self"}],"output":{"name":"error"}}],[8,"Visitor","","",null,null],[10,"start_element","","",12,{"inputs":[{"name":"self"},{"name":"source"},{"name":"configstring"}],"output":{"name":"result"}}],[10,"end_element","","",12,{"inputs":[{"name":"self"}],"output":{"name":"result"}}],[10,"set_table","","",12,{"inputs":[{"name":"self"},{"name":"source"},{"name":"span"}],"output":{"name":"result"}}],[10,"set_array","","",12,{"inputs":[{"name":"self"},{"name":"source"},{"name":"span"}],"output":{"name":"result"}}],[10,"append_string","","",12,{"inputs":[{"name":"self"},{"name":"source"},{"name":"configstring"}],"output":{"name":"result"}}],[10,"expand","","",12,{"inputs":[{"name":"self"},{"name":"source"},{"name":"configstring"}],"output":{"name":"result"}}]],"paths":[[3,"Token"],[3,"Lexer"],[3,"Error"],[3,"ConfigString"],[4,"StringQuoteType"],[4,"TokenKind"],[4,"ErrorKind"],[4,"StringKind"],[3,"Span"],[3,"Source"],[3,"Printer"],[8,"GetError"],[8,"Visitor"]]};
initSearch(searchIndex);
