#[derive(Default, Debug)]
pub struct Word<'a> {
    pub doc: &'a str,
    pub token: &'a str,
    pub stack: &'a str,
    pub help: &'a str,
}

pub struct Words<'a> {
    pub words: Vec<&'a Word<'a>>,
}

impl<'a> Word<'a> {
    pub fn documentation(&self) -> String {
        format!("# `{}`   `{}`\n\n{}", self.token, self.stack, self.help)
    }
}

impl Default for Words<'_> {
    fn default() -> Words<'static> {
        Words {
            words: vec![
                &Word {
                    doc: "/Store",
                    token: "!",
                    stack: "( x a-addr -- )",
                    help: "Store x at a-addr.",
                },

                &Word {
                    doc: "/num",
                    token: "#",
                    stack: "( ud1 -- ud2 )",
                    help: "Divide ud1 by the number in BASE giving the quotient ud2 and the remainder n. (n is the least significant digit of ud1.) Convert n to external form and add the resulting character to the beginning of the pictured numeric output string. An ambiguous condition exists if # executes outside of a <# #> delimited number conversion.",
                },

                &Word {
                    doc: "/num-end",
                    token: "#>",
                    stack: "( xd -- c-addr u )",
                    help: "Drop xd. Make the pictured numeric output string available as a character string. c-addr and u specify the resulting character string. A program may replace characters within the string.",
                },

                &Word {
                    doc: "/numS",
                    token: "#S",
                    stack: "( ud1 -- ud2 )",
                    help: "Convert one digit of ud1 according to the rule for #. Continue conversion until the quotient is zero. ud2 is zero. An ambiguous condition exists if #S executes outside of a <# #> delimited number conversion.",
                },

                &Word {
                    doc: "/Tick",
                    token: "'",
                    stack: "( '<spaces>name' -- xt )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Find name and return xt, the execution token for name. An ambiguous condition exists if name is not found. When interpreting, ' xyz EXECUTE is equivalent to xyz. Many Forth systems use a state-smart tick. Many do not. Forth-2012 follows the usage of Forth 94.",
                },

                &Word {
                    doc: "/p",
                    token: "(",
                    stack: "( 'ccc<paren>' -- )",
                    help: "Parse ccc delimited by ) (right parenthesis). ( is an immediate word.",
                },

                &Word {
                    doc: "/Times",
                    token: "*",
                    stack: "( n1 | u1 n2 | u2 -- n3 | u3 )",
                    help: "Multiply n1 | u1 by n2 | u2 giving the product n3 | u3.",
                },

                &Word {
                    doc: "/TimesDiv",
                    token: "*/",
                    stack: "( n1 n2 n3 -- n4 )",
                    help: "Multiply n1 by n2 producing the intermediate double-cell result d. Divide d by n3 giving the single-cell quotient n4. An ambiguous condition exists if n3 is zero or if the quotient n4 lies outside the range of a signed number. If d and n3 differ in sign, the implementation-defined result returned will be the same as that returned by either the phrase >R M* R> FM/MOD SWAP DROP or the phrase >R M* R> SM/REM SWAP DROP.",
                },

                &Word {
                    doc: "/TimesDivMOD",
                    token: "*/MOD",
                    stack: "( n1 n2 n3 -- n4 n5 )",
                    help: "Multiply n1 by n2 producing the intermediate double-cell result d. Divide d by n3 producing the single-cell remainder n4 and the single-cell quotient n5. An ambiguous condition exists if n3 is zero, or if the quotient n5 lies outside the range of a single-cell signed integer. If d and n3 differ in sign, the implementation-defined result returned will be the same as that returned by either the phrase >R M* R> FM/MOD or the phrase >R M* R> SM/REM.",
                },

                &Word {
                    doc: "/Plus",
                    token: "+",
                    stack: "( n1 | u1 n2 | u2 -- n3 | u3 )",
                    help: "Add n2 | u2 to n1 | u1, giving the sum n3 | u3.",
                },

                &Word {
                    doc: "/PlusStore",
                    token: "+!",
                    stack: "( n | u a-addr -- )",
                    help: "Add n | u to the single-cell number at a-addr.",
                },

                &Word {
                    doc: "/PlusLOOP",
                    token: "+LOOP",
                    stack: "( C: do-sys -- )",
                    help: "Append the run-time semantics given below to the current definition. Resolve the destination of all unresolved occurrences of LEAVE between the location given by do-sys and the next location for a transfer of control, to execute the words following +LOOP. An ambiguous condition exists if the loop control parameters are unavailable. Add n to the loop index. If the loop index did not cross the boundary between the loop limit minus one and the loop limit, continue execution at the beginning of the loop. Otherwise, discard the current loop control parameters and continue execution immediately following the loop.",
                },

                &Word {
                    doc: "/Comma",
                    token: ",",
                    stack: "( x -- )",
                    help: "Reserve one cell of data space and store x in the cell. If the data-space pointer is aligned when , begins execution, it will remain aligned when , finishes execution. An ambiguous condition exists if the data-space pointer is not aligned prior to execution of ,. See: 6.2.0945 COMPILE,.",
                },

                &Word {
                    doc: "/Minus",
                    token: "-",
                    stack: "( n1 | u1 n2 | u2 -- n3 | u3 )",
                    help: "Subtract n2 | u2 from n1 | u1, giving the difference n3 | u3.",
                },

                &Word {
                    doc: "/d",
                    token: ".",
                    stack: "( n -- )",
                    help: "Display n in free field format.",
                },

                &Word {
                    doc: "/Dotq",
                    token: ".\"",
                    stack: "( 'ccc<quote>' -- )",
                    help: "Parse ccc delimited by ' (double-quote). Append the run-time semantics given below to the current definition. Display ccc. An implementation may define interpretation semantics for .' if desired. In one plausible implementation, interpreting .' would display the delimited message. In another plausible implementation, interpreting .' would compile code to display the message later. In still another plausible implementation, interpreting .' would be treated as an exception. Given this variation a Standard Program may not use .' while interpreting. Similarly, a Standard Program may not compile POSTPONE .' inside a new word, and then use that word while interpreting. See F.6.1.1320 EMIT.",
                },

                &Word {
                    doc: "/Div",
                    token: "/",
                    stack: "( n1 n2 -- n3 )",
                    help: "Divide n1 by n2, giving the single-cell quotient n3. An ambiguous condition exists if n2 is zero. If n1 and n2 differ in sign, the implementation-defined result returned will be the same as that returned by either the phrase >R S>D R> FM/MOD SWAP DROP or the phrase >R S>D R> SM/REM SWAP DROP.",
                },

                &Word {
                    doc: "/DivMOD",
                    token: "/MOD",
                    stack: "( n1 n2 -- n3 n4 )",
                    help: "Divide n1 by n2, giving the single-cell remainder n3 and the single-cell quotient n4. An ambiguous condition exists if n2 is zero. If n1 and n2 differ in sign, the implementation-defined result returned will be the same as that returned by either the phrase >R S>D R> FM/MOD or the phrase >R S>D R> SM/REM.",
                },

                &Word {
                    doc: "/Zeroless",
                    token: "0<",
                    stack: "( n -- flag )",
                    help: "flag is true if and only if n is less than zero.",
                },

                &Word {
                    doc: "/ZeroEqual",
                    token: ":",
                    stack: "( x -- flag )",
                    help: "flag is true if and only if x is equal to zero.",
                },

                &Word {
                    doc: "/OnePlus",
                    token: "1+",
                    stack: "( n1 | u1 -- n2 | u2 )",
                    help: "Add one (1) to n1 | u1 giving the sum n2 | u2.",
                },

                &Word {
                    doc: "/OneMinus",
                    token: "1-",
                    stack: "( n1 | u1 -- n2 | u2 )",
                    help: "Subtract one (1) from n1 | u1 giving the difference n2 | u2.",
                },

                &Word {
                    doc: "/TwoStore",
                    token: "2!",
                    stack: "( x1 x2 a-addr -- )",
                    help: "Store the cell pair x1 x2 at a-addr, with x2 at a-addr and x1 at the next consecutive cell. It is equivalent to the sequence SWAP OVER ! CELL+ !.",
                },

                &Word {
                    doc: "/TwoTimes",
                    token: "2*",
                    stack: "( x1 -- x2 )",
                    help: "x2 is the result of shifting x1 one bit toward the most-significant bit, filling the vacated least-significant bit with zero.",
                },

                &Word {
                    doc: "/TwoDiv",
                    token: "2/",
                    stack: "( x1 -- x2 )",
                    help: "x2 is the result of shifting x1 one bit toward the least-significant bit, leaving the most-significant bit unchanged.",
                },

                &Word {
                    doc: "/TwoFetch",
                    token: "2@",
                    stack: "( a-addr -- x1 x2 )",
                    help: "Fetch the cell pair x1 x2 stored at a-addr. x2 is stored at a-addr and x1 at the next consecutive cell. It is equivalent to the sequence DUP CELL+ @ SWAP @.",
                },

                &Word {
                    doc: "/TwoDROP",
                    token: "2DROP",
                    stack: "( x1 x2 -- )",
                    help: "Drop cell pair x1 x2 from the stack.",
                },

                &Word {
                    doc: "/TwoDUP",
                    token: "2DUP",
                    stack: "( x1 x2 -- x1 x2 x1 x2 )",
                    help: "Duplicate cell pair x1 x2.",
                },

                &Word {
                    doc: "/TwoOVER",
                    token: "2OVER",
                    stack: "( x1 x2 x3 x4 -- x1 x2 x3 x4 x1 x2 )",
                    help: "Copy cell pair x1 x2 to the top of the stack.",
                },

                &Word {
                    doc: "/TwoSWAP",
                    token: "2SWAP",
                    stack: "( x1 x2 x3 x4 -- x3 x4 x1 x2 )",
                    help: "Exchange the top two cell pairs.",
                },

                &Word {
                    doc: "/Colon",
                    token: ":",
                    stack: "( C: '<spaces>name' -- colon-sys )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name, called a 'colon definition'. Enter compilation state and start the current definition, producing colon-sys. Append the initiation semantics given below to the current definition. Save implementation-dependent information nest-sys about the calling definition. The stack effects i * x represent arguments to name. Execute the definition name. The stack effects i * x and j * x represent arguments to and results from name, respectively. In Forth 83, this word was specified to alter the search order. This specification is explicitly removed in this standard. We believe that in most cases this has no effect; however, systems that allow many search orders found the Forth-83 behavior of colon very undesirable. The following tests the dictionary search order:",
                },

                &Word {
                    doc: "/Semi",
                    token: ";",
                    stack: "( C: colon-sys -- )",
                    help: "Append the run-time semantics below to the current definition. End the current definition, allow it to be found in the dictionary and enter interpretation state, consuming colon-sys. If the data-space pointer is not aligned, reserve enough data space to align it. Return to the calling definition specified by nest-sys. One function performed by both ; and ;CODE is to allow the current definition to be found in the dictionary. If the current definition was created by :NONAME the current definition has no definition name and thus cannot be found in the dictionary. If :NONAME is implemented the Forth compiler must maintain enough information about the current definition to allow ; and ;CODE to determine whether or not any action must be taken to allow it to be found.",
                },

                &Word {
                    doc: "/less",
                    token: "<",
                    stack: "( n1 n2 -- flag )",
                    help: "flag is true if and only if n1 is less than n2.",
                },

                &Word {
                    doc: "/num-start",
                    token: "<#",
                    stack: "( -- )",
                    help: "Initialize the pictured numeric output conversion process.",
                },

                &Word {
                    doc: "/Equal",
                    token: ":",
                    stack: "( x1 x2 -- flag )",
                    help: "flag is true if and only if x1 is bit-for-bit the same as x2.",
                },

                &Word {
                    doc: "/more",
                    token: ">",
                    stack: "( n1 n2 -- flag )",
                    help: "flag is true if and only if n1 is greater than n2.",
                },

                &Word {
                    doc: "/toBODY",
                    token: ">BODY",
                    stack: "( xt -- a-addr )",
                    help: "a-addr is the data-field address corresponding to xt. An ambiguous condition exists if xt is not for a word defined via CREATE.",
                },

                &Word {
                    doc: "/toIN",
                    token: ">IN",
                    stack: "( -- a-addr )",
                    help: "a-addr is the address of a cell containing the offset in characters from the start of the input buffer to the start of the parse area.",
                },

                &Word {
                    doc: "/toNUMBER",
                    token: ">NUMBER",
                    stack: "( ud1 c-addr1 u1 -- ud2 c-addr2 u2 )",
                    help: "ud2 is the unsigned result of converting the characters within the string specified by c-addr1 u1 into digits, using the number in BASE, and adding each into ud1 after multiplying ud1 by the number in BASE. Conversion continues left-to-right until a character that is not convertible, including any '+' or '-', is encountered or the string is entirely converted. c-addr2 is the location of the first unconverted character or the first character past the end of the string if the string was entirely converted. u2 is the number of unconverted characters in the string. An ambiguous condition exists if ud2 overflows during the conversion.",
                },

                &Word {
                    doc: "/toR",
                    token: ">R",
                    stack: "( x -- )",
                    help: "Move x to the return stack.",
                },

                &Word {
                    doc: "/qDUP",
                    token: "?DUP",
                    stack: "( x -- 0  |  x x )",
                    help: "Duplicate x if it is non-zero.",
                },

                &Word {
                    doc: "/Fetch",
                    token: "@",
                    stack: "( a-addr -- x )",
                    help: "x is the value stored at a-addr.",
                },

                &Word {
                    doc: "/ABORT",
                    token: "ABORT",
                    stack: "( i * x -- )",
                    help: "Empty the data stack and perform the function of QUIT, which includes emptying the return stack, without displaying a message.",
                },

                &Word {
                    doc: "/ABORTq",
                    token: "ABORT\"",
                    stack: "( 'ccc<quote>' -- )",
                    help: "Parse ccc delimited by a ' (double-quote). Append the run-time semantics given below to the current definition. Remove x1 from the stack. If any bit of x1 is not zero, display ccc and perform an implementation-defined abort sequence that includes the function of ABORT.",
                },

                &Word {
                    doc: "/ABS",
                    token: "ABS",
                    stack: "( n -- u )",
                    help: "u is the absolute value of n.",
                },

                &Word {
                    doc: "/ALIGN",
                    token: "ALIGN",
                    stack: "( -- )",
                    help: "If the data-space pointer is not aligned, reserve enough space to align it.",
                },

                &Word {
                    doc: "/ALIGNED",
                    token: "ALIGNED",
                    stack: "( addr -- a-addr )",
                    help: "a-addr is the first aligned address greater than or equal to addr.",
                },

                &Word {
                    doc: "/ALLOT",
                    token: "ALLOT",
                    stack: "( n -- )",
                    help: "If n is greater than zero, reserve n address units of data space. If n is less than zero, release | n | address units of data space. If n is zero, leave the data-space pointer unchanged.",
                },

                &Word {
                    doc: "/AND",
                    token: "AND",
                    stack: "( x1 x2 -- x3 )",
                    help: "x3 is the bit-by-bit logical 'and' of x1 with x2.",
                },

                &Word {
                    doc: "/BASE",
                    token: "BASE",
                    stack: "( -- a-addr )",
                    help: "a-addr is the address of a cell containing the current number-conversion radix {{2...36}}.",
                },

                &Word {
                    doc: "/BEGIN",
                    token: "BEGIN",
                    stack: "( C: -- dest )",
                    help: "Put the next location for a transfer of control, dest, onto the control flow stack. Append the run-time semantics given below to the current definition. Continue execution.    : X ... BEGIN ... test UNTIL ;",
                },

                &Word {
                    doc: "/BL",
                    token: "BL",
                    stack: "( -- char )",
                    help: "char is the character value for a space.",
                },

                &Word {
                    doc: "/CStore",
                    token: "C!",
                    stack: "( char c-addr -- )",
                    help: "Store char at c-addr. When character size is smaller than cell size, only the number of low-order bits corresponding to character size are transferred.",
                },

                &Word {
                    doc: "/CComma",
                    token: "C,",
                    stack: "( char -- )",
                    help: "Reserve space for one character in the data space and store char in the space. If the data-space pointer is character aligned when C, begins execution, it will remain character aligned when C, finishes execution. An ambiguous condition exists if the data-space pointer is not character-aligned prior to execution of C,.",
                },

                &Word {
                    doc: "/CFetch",
                    token: "C@",
                    stack: "( c-addr -- char )",
                    help: "Fetch the character stored at c-addr. When the cell size is greater than character size, the unused high-order bits are all zeroes.",
                },

                &Word {
                    doc: "/CELLPlus",
                    token: "CELL+",
                    stack: "( a-addr1 -- a-addr2 )",
                    help: "Add the size in address units of a cell to a-addr1, giving a-addr2.",
                },

                &Word {
                    doc: "/CELLS",
                    token: "CELLS",
                    stack: "( n1 -- n2 )",
                    help: "n2 is the size in address units of n1 cells.",
                },

                &Word {
                    doc: "/CHAR",
                    token: "CHAR",
                    stack: "( '<spaces>name' -- char )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Put the value of its first character onto the stack.",
                },

                &Word {
                    doc: "/CHARPlus",
                    token: "CHAR+",
                    stack: "( c-addr1 -- c-addr2 )",
                    help: "Add the size in address units of a character to c-addr1, giving c-addr2.",
                },

                &Word {
                    doc: "/CHARS",
                    token: "CHARS",
                    stack: "( n1 -- n2 )",
                    help: "n2 is the size in address units of n1 characters.",
                },

                &Word {
                    doc: "/CONSTANT",
                    token: "CONSTANT",
                    stack: "( x '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name with the execution semantics defined below. Place x on the stack.",
                },

                &Word {
                    doc: "/COUNT",
                    token: "COUNT",
                    stack: "( c-addr1 -- c-addr2 u )",
                    help: "Return the character string specification for the counted string stored at c-addr1. c-addr2 is the address of the first character after c-addr1. u is the contents of the character at c-addr1, which is the length in characters of the string at c-addr2.",
                },

                &Word {
                    doc: "/CR",
                    token: "CR",
                    stack: "( -- )",
                    help: "Cause subsequent output to appear at the beginning of the next line.",
                },

                &Word {
                    doc: "/CREATE",
                    token: "CREATE",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name with the execution semantics defined below. If the data-space pointer is not aligned, reserve enough data space to align it. The new data-space pointer defines name's data field. CREATE does not allocate data space in name's data field. a-addr is the address of name's data field. The execution semantics of name may be extended by using DOES>. Reservation of data field space is typically done with ALLOT.",
                },

                &Word {
                    doc: "/DECIMAL",
                    token: "DECIMAL",
                    stack: "( -- )",
                    help: "Set the numeric conversion radix to ten (decimal).",
                },

                &Word {
                    doc: "/DEPTH",
                    token: "DEPTH",
                    stack: "( -- +n )",
                    help: "+n is the number of single-cell values contained in the data stack before +n was placed on the stack.",
                },

                &Word {
                    doc: "/DO",
                    token: "DO",
                    stack: "( C: -- do-sys )",
                    help: "Place do-sys onto the control-flow stack. Append the run-time semantics given below to the current definition. The semantics are incomplete until resolved by a consumer of do-sys such as LOOP. Set up loop control parameters with index n2 | u2 and limit n1 | u1. An ambiguous condition exists if n1 | u1 and n2 | u2 are not both the same type. Anything already on the return stack becomes unavailable until the loop-control parameters are discarded.    : X ... limit first DO ... LOOP ;",
                },

                &Word {
                    doc: "/DOES",
                    token: "DOES>",
                    stack: "( C: colon-sys1 -- colon-sys2 )",
                    help: "Append the run-time semantics below to the current definition. Whether or not the current definition is rendered findable in the dictionary by the compilation of DOES> is implementation defined. Consume colon-sys1 and produce colon-sys2. Append the initiation semantics given below to the current definition. Replace the execution semantics of the most recent definition, referred to as name, with the name execution semantics given below. Return control to the calling definition specified by nest-sys1. An ambiguous condition exists if name was not defined with CREATE or a user-defined word that calls CREATE. Save implementation-dependent information nest-sys2 about the calling definition. Place name's data field address on the stack. The stack effects i * x represent arguments to name. Execute the portion of the definition that begins with the initiation semantics appended by the DOES> which modified name. The stack effects i * x and j * x represent arguments to and results from name, respectively. Following DOES>, a Standard Program may not make any assumptions regarding the ability to find either the name of the definition containing the DOES> or any previous definition whose name may be concealed by it. DOES> effectively ends one definition and begins another as far as local variables and control-flow structures are concerned. The compilation behavior makes it clear that the user is not entitled to place DOES> inside any control-flow structures.",
                },

                &Word {
                    doc: "/DROP",
                    token: "DROP",
                    stack: "( x -- )",
                    help: "Remove x from the stack.",
                },

                &Word {
                    doc: "/DUP",
                    token: "DUP",
                    stack: "( x -- x x )",
                    help: "Duplicate x.",
                },

                &Word {
                    doc: "/ELSE",
                    token: "ELSE",
                    stack: "( C: orig1 -- orig2 )",
                    help: "Put the location of a new unresolved forward reference orig2 onto the control flow stack. Append the run-time semantics given below to the current definition. The semantics will be incomplete until orig2 is resolved (e.g., by THEN). Resolve the forward reference orig1 using the location following the appended run-time semantics. Continue execution at the location given by the resolution of orig2.",
                },

                &Word {
                    doc: "/EMIT",
                    token: "EMIT",
                    stack: "( x -- )",
                    help: "If x is a graphic character in the implementation-defined character set, display x. The effect of EMIT for all other values of x is implementation-defined.",
                },

                &Word {
                    doc: "/ENVIRONMENTq",
                    token: "ENVIRONMENT?",
                    stack: "( c-addr u -- false  |  i * x true )",
                    help: "c-addr is the address of a character string and u is the string's character count. u may have a value in the range from zero to an implementation-defined maximum which shall not be less than 31. The character string should contain a keyword from 3.2.6 Environmental queries or the optional word sets to be checked for correspondence with an attribute of the present environment. If the system treats the attribute as unknown, the returned flag is false; otherwise, the flag is true and the i * x returned is of the type specified in the table for the attribute queried.",
                },

                &Word {
                    doc: "/EVALUATE",
                    token: "EVALUATE",
                    stack: "( i * x c-addr u -- j * x )",
                    help: "Save the current input source specification. Store minus-one (-1) in SOURCE-ID if it is present. Make the string described by c-addr and u both the input source and input buffer, set >IN to zero, and interpret. When the parse area is empty, restore the prior input source specification. Other stack effects are due to the words EVALUATEd.",
                },

                &Word {
                    doc: "/EXECUTE",
                    token: "EXECUTE",
                    stack: "( i * x xt -- j * x )",
                    help: "Remove xt from the stack and perform the semantics identified by it. Other stack effects are due to the word EXECUTEd.",
                },

                &Word {
                    doc: "/EXIT",
                    token: "EXIT",
                    stack: "( -- )",
                    help: "Return control to the calling definition specified by nest-sys. Before executing EXIT within a do-loop, a program shall discard the loop-control parameters by executing UNLOOP.",
                },

                &Word {
                    doc: "/FILL",
                    token: "FILL",
                    stack: "( c-addr u char -- )",
                    help: "If u is greater than zero, store char in each of u consecutive characters of memory beginning at c-addr.",
                },

                &Word {
                    doc: "/FIND",
                    token: "FIND",
                    stack: "( c-addr -- c-addr 0  |  xt 1  |  xt -1 )",
                    help: "Find the definition named in the counted string at c-addr. If the definition is not found, return c-addr and zero. If the definition is found, return its execution token xt. If the definition is immediate, also return one (1), otherwise also return minus-one (-1). For a given string, the values returned by FIND while compiling may differ from those returned while not compiling.",
                },

                &Word {
                    doc: "/FMDivMOD",
                    token: "FM/MOD",
                    stack: "( d1 n1 -- n2 n3 )",
                    help: "Divide d1 by n1, giving the floored quotient n3 and the remainder n2. Input and output stack arguments are signed. An ambiguous condition exists if n1 is zero or if the quotient lies outside the range of a single-cell signed integer. The committee considered providing two complete sets of explicitly named division operators, and declined to do so on the grounds that this would unduly enlarge and complicate the standard. Instead, implementors may define the normal division words in terms of either FM/MOD or SM/REM providing they document their choice. People wishing to have explicitly named sets of operators are encouraged to do so. FM/MOD may be used, for example, to define:",
                },

                &Word {
                    doc: "/HERE",
                    token: "HERE",
                    stack: "( -- addr )",
                    help: "addr is the data-space pointer.",
                },

                &Word {
                    doc: "/HOLD",
                    token: "HOLD",
                    stack: "( char -- )",
                    help: "Add char to the beginning of the pictured numeric output string. An ambiguous condition exists if HOLD executes outside of a <# #> delimited number conversion.",
                },

                &Word {
                    doc: "/I",
                    token: "I",
                    stack: "( -- n | u )",
                    help: "n | u is a copy of the current (innermost) loop index. An ambiguous condition exists if the loop control parameters are unavailable.",
                },

                &Word {
                    doc: "/IF",
                    token: "IF",
                    stack: "( C: -- orig )",
                    help: "Put the location of a new unresolved forward reference orig onto the control flow stack. Append the run-time semantics given below to the current definition. The semantics are incomplete until orig is resolved, e.g., by THEN or ELSE. If all bits of x are zero, continue execution at the location specified by the resolution of orig.    : X ... test IF ... THEN ... ; \\ Multiple ELSEs in an IF statement : melse IF 1 ELSE 2 ELSE 3 ELSE 4 ELSE 5 THEN ;",
                },

                &Word {
                    doc: "/IMMEDIATE",
                    token: "IMMEDIATE",
                    stack: "( -- )",
                    help: "Make the most recent definition an immediate word. An ambiguous condition exists if the most recent definition does not have a name or if it was defined as a SYNONYM.",
                },

                &Word {
                    doc: "/INVERT",
                    token: "INVERT",
                    stack: "( x1 -- x2 )",
                    help: "Invert all bits of x1, giving its logical inverse x2.",
                },

                &Word {
                    doc: "/J",
                    token: "J",
                    stack: "( -- n | u )",
                    help: "n | u is a copy of the next-outer loop index. An ambiguous condition exists if the loop control parameters of the next-outer loop, loop-sys1, are unavailable.    : X ... DO ... DO ... J ... LOOP ... +LOOP ... ;",
                },

                &Word {
                    doc: "/KEY",
                    token: "KEY",
                    stack: "( -- char )",
                    help: "Receive one character char, a member of the implementation-defined character set. Keyboard events that do not correspond to such characters are discarded until a valid character is received, and those events are subsequently unavailable. See A.10.6.2.1305 EKEY.",
                },

                &Word {
                    doc: "/LEAVE",
                    token: "LEAVE",
                    stack: "( -- )",
                    help: "Discard the current loop control parameters. An ambiguous condition exists if they are unavailable. Continue execution immediately following the innermost syntactically enclosing DO...LOOP or DO...+LOOP.    : X ... DO ... IF ... LEAVE THEN ... LOOP ... ;",
                },

                &Word {
                    doc: "/LITERAL",
                    token: "LITERAL",
                    stack: "( x -- )",
                    help: "Append the run-time semantics given below to the current definition. Place x on the stack.",
                },

                &Word {
                    doc: "/LOOP",
                    token: "LOOP",
                    stack: "( C: do-sys -- )",
                    help: "Append the run-time semantics given below to the current definition. Resolve the destination of all unresolved occurrences of LEAVE between the location given by do-sys and the next location for a transfer of control, to execute the words following the LOOP. An ambiguous condition exists if the loop control parameters are unavailable. Add one to the loop index. If the loop index is then equal to the loop limit, discard the loop parameters and continue execution immediately following the loop. Otherwise continue execution at the beginning of the loop.    : X ... limit first DO ... LOOP ... ;",
                },

                &Word {
                    doc: "/LSHIFT",
                    token: "LSHIFT",
                    stack: "( x1 u -- x2 )",
                    help: "Perform a logical left shift of u bit-places on x1, giving x2. Put zeroes into the least significant bits vacated by the shift. An ambiguous condition exists if u is greater than or equal to the number of bits in a cell.",
                },

                &Word {
                    doc: "/MTimes",
                    token: "M*",
                    stack: "( n1 n2 -- d )",
                    help: "d is the signed product of n1 times n2.",
                },

                &Word {
                    doc: "/MAX",
                    token: "MAX",
                    stack: "( n1 n2 -- n3 )",
                    help: "n3 is the greater of n1 and n2.",
                },

                &Word {
                    doc: "/MIN",
                    token: "MIN",
                    stack: "( n1 n2 -- n3 )",
                    help: "n3 is the lesser of n1 and n2.",
                },

                &Word {
                    doc: "/MOD",
                    token: "MOD",
                    stack: "( n1 n2 -- n3 )",
                    help: "Divide n1 by n2, giving the single-cell remainder n3. An ambiguous condition exists if n2 is zero. If n1 and n2 differ in sign, the implementation-defined result returned will be the same as that returned by either the phrase >R S>D R> FM/MOD DROP or the phrase >R S>D R> SM/REM DROP.",
                },

                &Word {
                    doc: "/MOVE",
                    token: "MOVE",
                    stack: "( addr1 addr2 u -- )",
                    help: "If u is greater than zero, copy the contents of u consecutive address units at addr1 to the u consecutive address units at addr2. After MOVE completes, the u consecutive address units at addr2 contain exactly what the u consecutive address units at addr1 contained before the move.",
                },

                &Word {
                    doc: "/NEGATE",
                    token: "NEGATE",
                    stack: "( n1 -- n2 )",
                    help: "Negate n1, giving its arithmetic inverse n2.",
                },

                &Word {
                    doc: "/OR",
                    token: "OR",
                    stack: "( x1 x2 -- x3 )",
                    help: "x3 is the bit-by-bit inclusive-or of x1 with x2.",
                },

                &Word {
                    doc: "/OVER",
                    token: "OVER",
                    stack: "( x1 x2 -- x1 x2 x1 )",
                    help: "Place a copy of x1 on top of the stack.",
                },

                &Word {
                    doc: "/POSTPONE",
                    token: "POSTPONE",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Find name. Append the compilation semantics of name to the current definition. An ambiguous condition exists if name is not found.    : ENDIF POSTPONE THEN ; IMMEDIATE",
                },

                &Word {
                    doc: "/QUIT",
                    token: "QUIT",
                    stack: "( -- )",
                    help: "Empty the return stack, store zero in SOURCE-ID if it is present, make the user input device the input source, and enter interpretation state. Do not display a message. Repeat the following:",
                },

                &Word {
                    doc: "/Rfrom",
                    token: "R>",
                    stack: "( -- x )",
                    help: "Move x from the return stack to the data stack.",
                },

                &Word {
                    doc: "/RFetch",
                    token: "R@",
                    stack: "( -- x )",
                    help: "Copy x from the return stack to the data stack.",
                },

                &Word {
                    doc: "/RECURSE",
                    token: "RECURSE",
                    stack: "( -- )",
                    help: "Append the execution semantics of the current definition to the current definition. An ambiguous condition exists if RECURSE appears in a definition after DOES>. This is Forth's recursion operator; in some implementations it is called MYSELF. The usual example is the coding of the factorial function. DECIMAL",
                },

                &Word {
                    doc: "/REPEAT",
                    token: "REPEAT",
                    stack: "( C: orig dest -- )",
                    help: "Append the run-time semantics given below to the current definition, resolving the backward reference dest. Resolve the forward reference orig using the location following the appended run-time semantics. Continue execution at the location given by dest.",
                },

                &Word {
                    doc: "/ROT",
                    token: "ROT",
                    stack: "( x1 x2 x3 -- x2 x3 x1 )",
                    help: "Rotate the top three stack entries.",
                },

                &Word {
                    doc: "/RSHIFT",
                    token: "RSHIFT",
                    stack: "( x1 u -- x2 )",
                    help: "Perform a logical right shift of u bit-places on x1, giving x2. Put zeroes into the most significant bits vacated by the shift. An ambiguous condition exists if u is greater than or equal to the number of bits in a cell.",
                },

                &Word {
                    doc: "/Sq",
                    help: "Parse ccc delimited by ' (double-quote). Append the run-time semantics given below to the current definition. Return c-addr and u describing a string consisting of the characters ccc. A program shall not alter the returned string. : GC5 S' A String\"2DROP ; \\ There is no space between the ' and 2DROP",
                    token: "S\"",
                    stack: "( 'ccc<quote>' -- )",
                },

                &Word {
                    doc: "/StoD",
                    token: "S>D",
                    stack: "( n -- d )",
                    help: "Convert the number n to the double-cell number d with the same numerical value.",
                },

                &Word {
                    doc: "/SIGN",
                    token: "SIGN",
                    stack: "( n -- )",
                    help: "If n is negative, add a minus sign to the beginning of the pictured numeric output string. An ambiguous condition exists if SIGN executes outside of a <# #> delimited number conversion.",
                },

                &Word {
                    doc: "/SMDivREM",
                    token: "SM/REM",
                    stack: "( d1 n1 -- n2 n3 )",
                    help: "Divide d1 by n1, giving the symmetric quotient n3 and the remainder n2. Input and output stack arguments are signed. An ambiguous condition exists if n1 is zero or if the quotient lies outside the range of a single-cell signed integer.",
                },

                &Word {
                    doc: "/SOURCE",
                    token: "SOURCE",
                    stack: "( -- c-addr u )",
                    help: "c-addr is the address of, and u is the number of characters in, the input buffer.",
                },

                &Word {
                    doc: "/SPACE",
                    token: "SPACE",
                    stack: "( -- )",
                    help: "Display one space.",
                },

                &Word {
                    doc: "/SPACES",
                    token: "SPACES",
                    stack: "( n -- )",
                    help: "If n is greater than zero, display n spaces.",
                },

                &Word {
                    doc: "/STATE",
                    token: "STATE",
                    stack: "( -- a-addr )",
                    help: "a-addr is the address of a cell containing the compilation-state flag. STATE is true when in compilation state, false otherwise. The true value in STATE is non-zero, but is otherwise implementation-defined. Only the following standard words alter the value in STATE: : (colon), ; (semicolon), ABORT, QUIT, :NONAME, [ (left-bracket), ] (right-bracket). STATE does not nest with text interpreter nesting. For example, the code sequence:",
                },

                &Word {
                    doc: "/SWAP",
                    token: "SWAP",
                    stack: "( x1 x2 -- x2 x1 )",
                    help: "Exchange the top two stack items.",
                },

                &Word {
                    doc: "/THEN",
                    token: "THEN",
                    stack: "( C: orig -- )",
                    help: "Append the run-time semantics given below to the current definition. Resolve the forward reference orig using the location of the appended run-time semantics. Continue execution.    : X ... test IF ... THEN ... ;",
                },

                &Word {
                    doc: "/TYPE",
                    token: "TYPE",
                    stack: "( c-addr u -- )",
                    help: "If u is greater than zero, display the character string specified by c-addr and u.",
                },

                &Word {
                    doc: "/Ud",
                    token: "U.",
                    stack: "( u -- )",
                    help: "Display u in free field format.",
                },

                &Word {
                    doc: "/Uless",
                    token: "U<",
                    stack: "( u1 u2 -- flag )",
                    help: "flag is true if and only if u1 is less than u2.",
                },

                &Word {
                    doc: "/UMTimes",
                    token: "UM*",
                    stack: "( u1 u2 -- ud )",
                    help: "Multiply u1 by u2, giving the unsigned double-cell product ud. All values and arithmetic are unsigned.",
                },

                &Word {
                    doc: "/UMDivMOD",
                    token: "UM/MOD",
                    stack: "( ud u1 -- u2 u3 )",
                    help: "Divide ud by u1, giving the quotient u3 and the remainder u2. All values and arithmetic are unsigned. An ambiguous condition exists if u1 is zero or if the quotient lies outside the range of a single-cell unsigned integer.",
                },

                &Word {
                    doc: "/UNLOOP",
                    token: "UNLOOP",
                    stack: "( -- )",
                    help: "Discard the loop-control parameters for the current nesting level. An UNLOOP is required for each nesting level before the definition may be EXITed. An ambiguous condition exists if the loop-control parameters are unavailable. UNLOOP allows the use of EXIT within the context of DO ... LOOP and related do-loop constructs. UNLOOP as a function has been called UNDO. UNLOOP is more indicative of the action: nothing gets undone — we simply stop doing it.",
                },

                &Word {
                    doc: "/UNTIL",
                    token: "UNTIL",
                    stack: "( C: dest -- )",
                    help: "Append the run-time semantics given below to the current definition, resolving the backward reference dest. If all bits of x are zero, continue execution at the location specified by dest.",
                },

                &Word {
                    doc: "/VARIABLE",
                    token: "VARIABLE",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name with the execution semantics defined below. Reserve one cell of data space at an aligned address. a-addr is the address of the reserved cell. A program is responsible for initializing the contents of the reserved cell.",
                },

                &Word {
                    doc: "/WHILE",
                    token: "WHILE",
                    stack: "( C: dest -- orig dest )",
                    help: "Put the location of a new unresolved forward reference orig onto the control flow stack, under the existing dest. Append the run-time semantics given below to the current definition. The semantics are incomplete until orig and dest are resolved (e.g., by REPEAT). If all bits of x are zero, continue execution at the location specified by the resolution of orig.",
                },

                &Word {
                    doc: "/WORD",
                    token: "WORD",
                    stack: "( char '<chars>ccc<char>' -- c-addr )",
                    help: "Skip leading delimiters. Parse characters ccc delimited by char. An ambiguous condition exists if the length of the parsed string is greater than the implementation-defined length of a counted string.",
                },

                &Word {
                    doc: "/XOR",
                    token: "XOR",
                    stack: "( x1 x2 -- x3 )",
                    help: "x3 is the bit-by-bit exclusive-or of x1 with x2.",
                },

                &Word {
                    doc: "/Bracket",
                    token: "[",
                    stack: "( -- )",
                    help: "Enter interpretation state. [ is an immediate word.",
                },

                &Word {
                    doc: "/BracketTick",
                    token: "[']",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Find name. Append the run-time semantics given below to the current definition. Place name's execution token xt on the stack. The execution token returned by the compiled phrase '['] X' is the same value returned by '' X' outside of compilation state. See: A.6.1.1550 FIND.",
                },

                &Word {
                    doc: "/BracketCHAR",
                    token: "[CHAR]",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Append the run-time semantics given below to the current definition. Place char, the value of the first character of name, on the stack.",
                },

                &Word {
                    doc: "/Dotp",
                    token: ".(",
                    stack: "( 'ccc<paren>' -- )",
                    help: "Parse and display ccc delimited by ) (right parenthesis). .( is an immediate word.",
                },

                &Word {
                    doc: "/DotR",
                    token: ".R",
                    stack: "( n1 n2 -- )",
                    help: "Display n1 right aligned in a field n2 characters wide. If the number of characters required to display n1 is greater than n2, all digits are displayed with no leading spaces in a field as wide as necessary.",
                },

                &Word {
                    doc: "/Zerone",
                    token: "0<>",
                    stack: "( x -- flag )",
                    help: "flag is true if and only if x is not equal to zero.",
                },

                &Word {
                    doc: "/Zeromore",
                    token: "0>",
                    stack: "( n -- flag )",
                    help: "flag is true if and only if n is greater than zero.",
                },

                &Word {
                    doc: "/TwotoR",
                    token: "2>R",
                    stack: "( x1 x2 -- )",
                    help: "Transfer cell pair x1 x2 to the return stack. Semantically equivalent to SWAP >R >R.",
                },

                &Word {
                    doc: "/TwoRfrom",
                    token: "2R>",
                    stack: "( -- x1 x2 )",
                    help: "Transfer cell pair x1 x2 from the return stack. Semantically equivalent to R> R> SWAP.",
                },

                &Word {
                    doc: "/TwoRFetch",
                    token: "2R@",
                    stack: "( -- x1 x2 )",
                    help: "Copy cell pair x1 x2 from the return stack. Semantically equivalent to R> R> 2DUP >R >R SWAP.",
                },

                &Word {
                    doc: "/ColonNONAME",
                    token: ":NONAME",
                    stack: "( C: -- colon-sys )",
                    help: "Create an execution token xt, enter compilation state and start the current definition, producing colon-sys. Append the initiation semantics given below to the current definition. Save implementation-dependent information nest-sys about the calling definition. The stack effects i * x represent arguments to xt. Execute the definition specified by xt. The stack effects i * x and j * x represent arguments to and results from xt, respectively.    DEFER print    :NONAME ( n -- ) . ; IS print",
                },

                &Word {
                    doc: "/ne",
                    token: "<>",
                    stack: "( x1 x2 -- flag )",
                    help: "flag is true if and only if x1 is not bit-for-bit the same as x2.",
                },

                &Word {
                    doc: "/qDO",
                    token: "?DO",
                    stack: "( C: -- do-sys )",
                    help: "Put do-sys onto the control-flow stack. Append the run-time semantics given below to the current definition. The semantics are incomplete until resolved by a consumer of do-sys such as LOOP. If n1 | u1 is equal to n2 | u2, continue execution at the location given by the consumer of do-sys. Otherwise set up loop control parameters with index n2 | u2 and limit n1 | u1 and continue executing immediately following ?DO. Anything already on the return stack becomes unavailable until the loop control parameters are discarded. An ambiguous condition exists if n1 | u1 and n2 | u2 are not both of the same type.    : X ... ?DO ... LOOP ... ;",
                },

                &Word {
                    doc: "/ACTION-OF",
                    token: "ACTION-OF",
                    stack: "( '<spaces>name' -- xt )",
                    help: "Skip leading spaces and parse name delimited by a space. xt is the execution token that name is set to execute. An ambiguous condition exists if name was not defined by DEFER, or if the name has not been set to execute an xt. Skip leading spaces and parse name delimited by a space. Append the run-time semantics given below to the current definition. An ambiguous condition exists if name was not defined by DEFER. xt is the execution token that name is set to execute. An ambiguous condition exists if name has not been set to execute an xt.",
                },

                &Word {
                    doc: "/AGAIN",
                    token: "AGAIN",
                    stack: "( C: dest -- )",
                    help: "Append the run-time semantics given below to the current definition, resolving the backward reference dest. Continue execution at the location specified by dest. If no other control flow words are used, any program code after AGAIN will not be executed. Unless word-sequence has a way to terminate, this is an endless loop.",
                },

                &Word {
                    doc: "/BUFFERColon",
                    token: "BUFFER:",
                    stack: "( u '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name, with the execution semantics defined below. Reserve u address units at an aligned address. Contiguity of this region with any other region is undefined. a-addr is the address of the space reserved by BUFFER: when it defined name. The program is responsible for initializing the contents.",
                },

                &Word {
                    doc: "/Cq",
                    token: "C\"",
                    stack: "( 'ccc<quote>' -- )",
                    help: "Parse ccc delimited by ' (double-quote) and append the run-time semantics given below to the current definition. Return c-addr, a counted string consisting of the characters ccc. A program shall not alter the returned string. See: A.3.1.3.4 Counted strings.",
                },

                &Word {
                    doc: "/CASE",
                    token: "CASE",
                    stack: "( C: -- case-sys )",
                    help: "Mark the start of the CASE...OF...ENDOF...ENDCASE structure. Append the run-time semantics given below to the current definition. Continue execution.",
                },

                &Word {
                    doc: "/COMPILEComma",
                    token: "COMPILE,",
                    stack: "( xt -- )",
                    help: "Append the execution semantics of the definition represented by xt to the execution semantics of the current definition. In traditional threaded-code implementations, compilation is performed by , (comma). This usage is not portable; it doesn't work for subroutine-threaded, native code, or relocatable implementations. Use of COMPILE, is portable.",
                },

                &Word {
                    doc: "/DEFER",
                    token: "DEFER",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name with the execution semantics defined below. Execute the xt that name is set to execute. An ambiguous condition exists if name has not been set to execute an xt.",
                },

                &Word {
                    doc: "/DEFERStore",
                    token: "DEFER!",
                    stack: "( xt2 xt1 -- )",
                    help: "Set the word xt1 to execute xt2. An ambiguous condition exists if xt1 is not for a word defined by DEFER.",
                },

                &Word {
                    doc: "/DEFERFetch",
                    token: "DEFER@",
                    stack: "( xt1 -- xt2 )",
                    help: "xt2 is the execution token xt1 is set to execute. An ambiguous condition exists if xt1 is not the execution token of a word defined by DEFER, or if xt1 has not been set to execute an xt.",
                },

                &Word {
                    doc: "/ENDCASE",
                    token: "ENDCASE",
                    stack: "( C: case-sys -- )",
                    help: "Mark the end of the CASE...OF...ENDOF...ENDCASE structure. Use case-sys to resolve the entire structure. Append the run-time semantics given below to the current definition. Discard the case selector x and continue execution.",
                },

                &Word {
                    doc: "/ENDOF",
                    token: "ENDOF",
                    stack: "( C: case-sys1 of-sys -- case-sys2 )",
                    help: "Mark the end of the OF...ENDOF part of the CASE structure. The next location for a transfer of control resolves the reference given by of-sys. Append the run-time semantics given below to the current definition. Replace case-sys1 with case-sys2 on the control-flow stack, to be resolved by ENDCASE. Continue execution at the location specified by the consumer of case-sys2.",
                },

                &Word {
                    doc: "/ERASE",
                    token: "ERASE",
                    stack: "( addr u -- )",
                    help: "If u is greater than zero, clear all bits in each of u consecutive address units of memory beginning at addr.",
                },

                &Word {
                    doc: "/FALSE",
                    token: "FALSE",
                    stack: "( -- false )",
                    help: "Return a false flag.",
                },

                &Word {
                    doc: "/HEX",
                    token: "HEX",
                    stack: "( -- )",
                    help: "Set contents of BASE to sixteen.",
                },

                &Word {
                    doc: "/HOLDS",
                    token: "HOLDS",
                    stack: "( c-addr u -- )",
                    help: "Adds the string represented by c-addr u to the pictured numeric output string. An ambiguous condition exists if HOLDS executes outside of a <# #> delimited number conversion.",
                },

                &Word {
                    doc: "/IS",
                    token: "IS",
                    stack: "( xt '<spaces>name' -- )",
                    help: "Skip leading spaces and parse name delimited by a space. Set name to execute xt. Skip leading spaces and parse name delimited by a space. Append the run-time semantics given below to the current definition. An ambiguous condition exists if name was not defined by DEFER. Set name to execute xt.",
                },

                &Word {
                    doc: "/MARKER",
                    token: "MARKER",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name with the execution semantics defined below. Restore all dictionary allocation and search order pointers to the state they had just prior to the definition of name. Remove the definition of name and all subsequent definitions. Restoration of any structures still existing that could refer to deleted definitions or deallocated data space is not necessarily provided. No other contextual information such as numeric base is affected.",
                },

                &Word {
                    doc: "/NIP",
                    token: "NIP",
                    stack: "( x1 x2 -- x2 )",
                    help: "Drop the first item below the top of stack.",
                },

                &Word {
                    doc: "/OF",
                    token: "OF",
                    stack: "( C: -- of-sys )",
                    help: "Put of-sys onto the control flow stack. Append the run-time semantics given below to the current definition. The semantics are incomplete until resolved by a consumer of of-sys such as ENDOF. If the two values on the stack are not equal, discard the top value and continue execution at the location specified by the consumer of of-sys, e.g., following the next ENDOF. Otherwise, discard both values and continue execution in line.",
                },

                &Word {
                    doc: "/PAD",
                    token: "PAD",
                    stack: "( -- c-addr )",
                    help: "c-addr is the address of a transient region that can be used to hold data for intermediate processing.",
                },

                &Word {
                    doc: "/PARSE",
                    token: "PARSE",
                    stack: "( char 'ccc<char>' -- c-addr u )",
                    help: "Parse ccc delimited by the delimiter char. The traditional Forth word for parsing is WORD. PARSE solves the following problems with WORD:",
                },

                &Word {
                    doc: "/PARSE-NAME",
                    token: "PARSE-NAME",
                    stack: "( '<spaces>name<space>' -- c-addr u )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. : isnotspace? ( c -- f )    isspace? : ; \\ test empty parse area     \\ line with white space",
                },

                &Word {
                    doc: "/PICK",
                    token: "PICK",
                    stack: "( xu...x1 x0 u -- xu...x1 x0 xu )",
                    help: "Remove u. Copy the xu to the top of the stack. An ambiguous condition exists if there are less than u+2 items on the stack before PICK is executed.",
                },

                &Word {
                    doc: "/REFILL",
                    token: "REFILL",
                    stack: "( -- flag )",
                    help: "Attempt to fill the input buffer from the input source, returning a true flag if successful.",
                },

                &Word {
                    doc: "/RESTORE-INPUT",
                    token: "RESTORE-INPUT",
                    stack: "( xn ... x1 n -- flag )",
                    help: "Attempt to restore the input source specification to the state described by x1 through xn. flag is true if the input source specification cannot be so restored.",
                },

                &Word {
                    doc: "/ROLL",
                    token: "ROLL",
                    stack: "( xu xu-1 ... x0 u -- xu-1 ... x0 xu )",
                    help: "Remove u. Rotate u+1 items on the top of the stack. An ambiguous condition exists if there are less than u+2 items on the stack before ROLL is executed.",
                },

                &Word {
                    doc: "/Seq",
                    token: "S\"",
                    stack: "( 'ccc<quote>' -- )",
                    help: "Parse ccc delimited by \" (double-quote), using the translation rules below. Append the run-time semantics given below to the current definition.",
                },

                &Word {
                    doc: "/SAVE-INPUT",
                    token: "SAVE-INPUT",
                    stack: "( -- xn ... x1 n )",
                    help: "x1 through xn describe the current state of the input source specification for later use by RESTORE-INPUT. SAVE-INPUT and RESTORE-INPUT are intended for repositioning within a single input source; for example, the following scenario is NOT allowed for a Standard Program:",
                },

                &Word {
                    doc: "/SOURCE-ID",
                    token: "SOURCE-ID",
                    stack: "( -- 0  |  -1  )",
                    help: "Identifies the input source as follows:",
                },

                &Word {
                    doc: "/TO",
                    token: "TO",
                    stack: "( i * x '<spaces>name' -- )",
                    help: "Skip leading spaces and parse name delimited by a space. Perform the 'TO name run-time' semantics given in the definition for the defining word of name. An ambiguous condition exists if name was not defined by a word with 'TO name run-time' semantics. Skip leading spaces and parse name delimited by a space. Append the 'TO name run-time' semantics given in the definition for the defining word	of name to the current definition. An ambiguous condition exists if name was not defined by a word with 'TO name run-time' semantics. Some implementations of TO do not parse; instead they set a mode flag that is tested by the subsequent execution of name. Standard programs must use TO as if it parses. Therefore TO and name must be contiguous and on the same line in the source text.",
                },

                &Word {
                    doc: "/TRUE",
                    token: "TRUE",
                    stack: "( -- true )",
                    help: "Return a true flag, a single-cell value with all bits set.",
                },

                &Word {
                    doc: "/TUCK",
                    token: "TUCK",
                    stack: "( x1 x2 -- x2 x1 x2 )",
                    help: "Copy the first (top) stack item below the second stack item.",
                },

                &Word {
                    doc: "/UDotR",
                    token: "U.R",
                    stack: "( u n -- )",
                    help: "Display u right aligned in a field n characters wide. If the number of characters required to display u is greater than n, all digits are displayed with no leading spaces in a field as wide as necessary.",
                },

                &Word {
                    doc: "/Umore",
                    token: "U>",
                    stack: "( u1 u2 -- flag )",
                    help: "flag is true if and only if u1 is greater than u2.",
                },

                &Word {
                    doc: "/UNUSED",
                    token: "UNUSED",
                    stack: "( -- u )",
                    help: "u is the amount of space remaining in the region addressed by HERE, in address units.",
                },

                &Word {
                    doc: "/VALUE",
                    token: "VALUE",
                    stack: "( x '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Create a definition for name with the execution semantics defined below, with an initial value equal to x. Place x on the stack. The value of x is that given when name was created, until the phrase x TO name is executed, causing a new value of x to be assigned to name. Assign the value x to name.",
                },

                &Word {
                    doc: "/WITHIN",
                    token: "WITHIN",
                    stack: "( n1 | u1 n2 | u2 n3 | u3 -- flag )",
                    help: "Perform a comparison of a test value n1 | u1 with a lower limit n2 | u2 and an upper limit n3 | u3, returning true if either (n2 | u2 < n3 | u3 and (n2 | u2 : n1 | u1 and n1 | u1 < n3 | u3)) or (n2 | u2 > n3 | u3 and (n2 | u2 : n1 | u1 or n1 | u1 < n3 | u3)) is true, returning false otherwise. An ambiguous condition exists n1 | u1, n2 | u2, and n3 | u3 are not all the same type.    33000 32000 34000 WITHIN",
                },

                &Word {
                    doc: "/BracketCOMPILE",
                    token: "[COMPILE]",
                    stack: "( '<spaces>name' -- )",
                    help: "Skip leading space delimiters. Parse name delimited by a space. Find name. If name has other than default compilation semantics, append them to the current definition; otherwise append the execution semantics of name. An ambiguous condition exists if name is not found. With an immediate word",
                },

                &Word {
                    doc: "/bs",
                    token: "\\",
                    stack: "( 'ccc<eol>' -- )",
                    help: "Parse and discard the remainder of the parse area. \\ is an immediate word.",
                },

                &Word {
                    doc: "/ACCEPT",
                    token: "ACCEPT",
                    stack: "( c-addr +n1 -- +n2 )",
                    help: "Receive a string of at most +n1 characters. An ambiguous condition exists if +n1 is zero or greater than 32,767. Display graphic characters as they are received. A program that depends on the presence or absence of non-graphic characters in the string has an environmental dependency. The editing functions, if any, that the system performs in order to construct the string are implementation-defined",
                },
            ],
        }
    }
}
