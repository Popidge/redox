# Iron Language Grammar Specification

## Overview

Iron is a verbose, lexically-expanded superset of Rust designed for optimal tokenization by Large Language Models. This document defines the formal grammar of the Iron language for parsing and transpilation purposes.

## Lexical Structure

### Keywords

Iron uses lowercase snake_case English phrases exclusively. All keywords are reserved and cannot be used as identifiers without the `user_` prefix.

#### Type Keywords
- `type`, `reference`, `mutable`, `raw`, `pointer`
- `optional`, `result`, `list`, `box`
- `tuple`, `array`, `slice`

#### Control Flow
- `if`, `condition`, `then`, `otherwise`, `end if`
- `compare`, `case` (for match)
- `while`, `repeat`, `for each`, `in`, `iterator`
- `loop`, `forever`, `exit`, `continue`
- `return`

#### Functions
- `function`, `with`, `generic`, `implementing`
- `takes`, `parameter`, `returns`
- `begin`, `end function`
- `call`, `method`, `on`, `with` (for arguments)

#### Bindings
- `define`, `as`, `set`, `equal to`
- `constant`, `static`

#### Structs and Enums
- `structure`, `fields`, `field`, `end structure`
- `enumeration`, `variants`, `variant`, `end enumeration`
- `of` (for types)

#### Special Values
- `context` (self), `some`, `none`, `ok`, `error`

## Syntax

### Functions

```
function <name> [with generic type <T> [implementing <bound>]]
    [takes <param> of <type> [and <param> of <type>...]]
    [returns <type>]
begin
    <statements>
end function
```

### Variable Bindings

```
define [mutable] <name> as <expression>
set <name> equal to <expression>
```

### Types

- `type T` -> T
- `reference to T` -> &T
- `mutable reference to T` -> &mut T
- `optional T` -> Option<T>
- `result of T or error E` -> Result<T, E>
- `list of T` -> Vec<T>
- `box containing T` -> Box<T>
- `function taking A returning B` -> fn(A) -> B

### Control Flow

#### If Statement
```
if <condition> then
begin
    <statements>
end if
[otherwise
begin
    <statements>
end if]
```

#### For Loop
```
for each <var> in <iterator> repeat
begin
    <statements>
end for
```

#### While Loop
```
while <condition> repeat
begin
    <statements>
end while
```

#### Match Expression
```
compare <expression>
    case <pattern> then <expression>
    ...
end compare
```

### Structs

```
structure <name> [with generic type <T>] with fields
    <field> of <type>
    ...
end structure
```

### Enums

```
enumeration <name> [with generic type <T>] with variants
    <variant>
    <variant> of <type>
    <variant> with <field> of <type> and <field> of <type>
    ...
end enumeration
```

### Expressions

#### Method Calls
```
call method <name> on <receiver> [with <args>]
```

#### Function Calls
```
call <name> with <args>
```

#### Binary Operations
```
<left> <operator> <right>
```

Where operators are:
- `plus` -> +
- `minus` -> -
- `times` -> *
- `divided by` -> /
- `and` -> &&
- `or` -> ||
- `equal to` -> ==
- `not equal to` -> !=
- `less than` -> <
- `greater than` -> >
- `less than or equal to` -> <=
- `greater than or equal to` -> >=

#### Try Operator
```
<expression> unwrap or return error
```

## Comments

```
note that <comment text>
```

## Indentation

Iron uses 4-space indentation for block contents. Blocks are delimited by:
- Opening: `begin` or keyword like `then`, `repeat`
- Closing: `end <keyword>`

## Parsing Strategy

1. Tokenize input into words and delimiters
2. Use recursive descent parsing based on the grammar above
3. Build AST representing the program structure
4. Generate Rust code from the AST
