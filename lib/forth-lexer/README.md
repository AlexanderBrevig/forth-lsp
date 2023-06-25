# Forth Lexer

Given the forth program:

```forth
: add1 ( n -- n )
  1 +  \ adds one
;
```

Here's the output you'll get from this parser (an excerpt from our tests in parser.rs):

```rust
let mut lexer = Lexer::new(": add1 ( n -- n )\n  1 + \\ adds one\n;");
let tokens = lexer.parse();
let expected = vec![
    // Notice the data has two fields, start and end
    // This is the index into the string
    Colon(Data::new(0, 0, ':')),
    Word(Data::new(2, 6, "add1".into())),
    Comment(Data::new(7, 17, "( n -- n )".into())),
    Number(Data::new(20, 21, "1".into())),
    Word(Data::new(22, 23, "+".into())),
    Comment(Data::new(24, 34, "\\ adds one".into())),
    Semicolon(Data::new(35, 36, ';')),
];
assert_eq!(tokens, expected)
```

If you use `ropey` you can get the actual slice for a token by

```rust
let progn = "word1 word2 word3";
let rope = ropey::Rope::from_str(progn);
let mut lexer = Lexer::new(progn);
let tokens = lexer.parse();
// Let's get the `Data<String>` second `Word` from the list
let word2 = if let Some(Token::Word(word)) = tokens.get(1) { word.to_owned() } else { Data::<String>::default() };
let x = rope.slice(&word2); // Data implements RangeBounds
assert_eq!("word2", word2.value);
assert_eq!(word2.value, x);
```
