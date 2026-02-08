**Product Requirements Document: Redox Transpiler v0.1 (Iron Reduction)**

---

## 1. Executive Summary

**Redox** is a deterministic source-to-source transpiler that converts valid Rust code into **Iron**, a verbose, lexically-expanded superset of Rust designed for optimal tokenisation by Large Language Models (LLMs).

Iron represents every Rust primitive (ownership operators, type annotations, control flow symbols) as high-probability English vocabulary rather than punctuation-heavy sigils. This creates a 1:1 semantic mapping where each Iron construct unambiguously represents exactly one Rust AST node, reducing token entropy and improving LLM comprehension of code structure.

**Chemical Metaphor**: Just as reduction adds electrons to a metal, Redox "adds lexical electrons" to Rust, expanding compact syntax into verbose, semantically-rich Iron.

---

## 2. Goals

### 2.1 Primary Goal
Create a **deterministic, lossless transpiler** (Redox) capable of parsing any valid Rust source file and emitting equivalent Iron code with **unambiguous 1:1 AST correspondence**.

### 2.2 Success Metrics
- Every valid Rust file transpiles to Iron without compilation errors
- Iron output uses exclusively alphanumeric tokens and whitespace (minimising punctuation sigils: `&`, `->`, `<`, `>`, `::`, etc.)
- Transpilation is deterministic: identical Rust input produces identical Iron output (modulo comments)
- Round-trip property: Iron → Rust → Iron produces semantically identical results (phase 2)

---

## 3. Rationale & Technical Justification

### 3.1 The Token Problem
Current LLM tokenisers fragment Rust syntax inefficiently:
- `&mut self` → 3-4 tokens (`&`, `mut`, `self` or subword splits)
- `-> Option<T>` → 6+ tokens including fragmentary symbols
- `Vec::new()` → splits on `::` creating non-semantic boundaries

These low-probability symbols create "attention noise"—neural pathways must learn that `->` means "returns" and `<>` delimits generics, despite these being rare in natural language pre-training.

### 3.2 The Iron Solution
Iron expands these into high-probability semantic tokens:
- `&mut self` → `mutable reference to context` (5 common English words)
- `-> Option<T>` → `returns optional of type T` (clear semantic mapping)
- `Vec::new()` → `create new vector` (imperative verb phrase)

**Benefits:**
- **Semantic Transparency**: LLMs understand "returns" better than `->`
- **Reduced Token Count**: Verbose text often tokenises more efficiently than punctuation sequences
- **Deterministic Parsing**: No syntactic ambiguity—every keyword maps to exactly one Rust semantic construct

---

## 4. Functional Requirements

### 4.1 Core Transpilation Engine
**Must support:**
- Function definitions (including generics, trait bounds, async)
- Variable bindings (`let`, `const`, `static`) with mutability
- All ownership types: owned values, shared references (`&`), mutable references (`&mut`), raw pointers
- Composite types: structs, enums, tuples, arrays, slices
- Control flow: `if`/`else`, `match`, `while`, `for`, `loop`, `break`, `continue`, `return`
- Type signatures: generics, lifetimes (explicit), trait bounds, return types
- Method calls and associated functions
- Standard library types: `Option`, `Result`, `Vec`, `HashMap`, `String`, `Box`, `Rc`, `Arc`
- Error handling: `?` operator, `Result` matching
- Closures and function types
- Modules and use statements (simplified)

### 4.2 The Iron Specification (v0.1)

#### 4.2.1 Naming Convention
All Iron keywords use **lowercase snake_case English phrases**. No abbreviations. No symbols.

#### 4.2.2 Type Representations
| Rust | Iron |
|------|------|
| `T` | `type T` |
| `&T` | `reference to T` |
| `&mut T` | `mutable reference to T` |
| `*const T` | `raw pointer to T` |
| `*mut T` | `mutable raw pointer to T` |
| `-> T` | `returns T` |
| `Option<T>` | `optional T` |
| `Result<T, E>` | `result of T or error E` |
| `Vec<T>` | `list of T` |
| `Box<T>` | `box containing T` |
| `fn(A) -> B` | `function taking A returning B` |

#### 4.2.3 Ownership and Binding
| Rust | Iron |
|------|------|
| `let x = val;` | `define x as val` |
| `let mut x = val;` | `define mutable x as val` |
| `mut param` | `mutable parameter` |
| `self` | `context` |
| `&self` | `reference to context` |
| `&mut self` | `mutable reference to context` |

#### 4.2.4 Control Flow
| Rust | Iron |
|------|------|
| `if cond { }` | `if condition then begin end` |
| `else { }` | `otherwise begin end` |
| `match expr { }` | `compare expr begin end` |
| `Pat => expr` | `case Pat then expr` |
| `while cond { }` | `while condition repeat begin end` |
| `for x in iter { }` | `for each x in iterator begin begin end` |
| `loop { }` | `repeat forever begin end` |
| `break` | `exit loop` |
| `continue` | `continue loop` |
| `return x` | `return x` |

#### 4.2.5 Functions
**Format:**
```
function name with generic type T implementing Bound
    takes parameter1 of Type1 and parameter2 of Type2
    returns ReturnType
begin
    body
end function
```

#### 4.2.6 Structs and Enums
**Struct:**
```
structure Name with fields
    field1 of Type1
    field2 of Type2
end structure
```

**Enum:**
```
enumeration Name with variants
    Variant1
    Variant2 of Type
    Variant3 with fields field1 of Type1 and field2 of Type2
end enumeration
```

#### 4.2.7 Method Calls
| Rust | Iron |
|------|------|
| `obj.method(args)` | `call method on obj with args` |
| `Struct::method(args)` | `call associated method on Struct with args` |
| `self.field` | `field of context` |

### 4.3 CLI Interface
```bash
redox reduce <input.rs> [--output <output.iron>]
```
- Input: Valid Rust source file (`.rs`)
- Output: Iron source file (`.iron`)
- Exit code 0 on success, non-zero on parse/transpile failure
- STDERR: Parse errors or unsupported syntax warnings
- STDOUT: Iron code (if no output file specified)

---

## 5. Technical Architecture

### 5.1 Dependencies
- **syn** (v2.x): Rust parsing (full feature set)
- **proc-macro2**: Token handling
- **quote**: Code generation utilities (optional, for Rust output later)
- **clap**: CLI argument parsing

### 5.2 Implementation Strategy
1. **AST Visitor**: Implement `syn::Visit` trait to traverse Rust AST
2. **Token Emitter**: Map each AST node type to Iron vocabulary via pattern matching
3. **Formatting**: Indentation-based blocks (4 spaces), newline-delimited
4. **Error Handling**: Panic on unsupported syntax with descriptive message (MVP quality)

### 5.3 File Structure
```
redox/
├── src/
│   ├── main.rs          # CLI entry
│   ├── lib.rs           # Public API
│   ├── parser.rs        # syn integration
│   ├── emitter.rs       # Iron generation logic
│   └── mappings.rs      # Rust↔Iron dictionary
└── Cargo.toml
```

---

## 6. Constraints & Non-Goals

### 6.1 Explicitly Out of Scope (v0.1)
- **Iron → Rust transpilation** (Oxidation): Phase 2 requirement, not needed now
- **Macro expansion**: Handle `macro_rules!` definitions by emitting placeholder text: `macro definition not expanded`
- **Procedural macros**: Skip or placeholder
- **Unsafe blocks**: Transpile literally but mark with `unsafe block begin`
- **Semantic validation**: No embedding-based verification (Phase 3)
- **Formatting preservation**: Iron output is canonical, not style-preserving
- **Comments**: May be stripped or converted to Iron comments (`note that ...`)

### 6.2 Limitations Accepted
- Iron output will be verbose (this is the point)
- Transpilation is one-way for this phase
- Not all Rust edge cases (const generics, GATs) need perfect formatting, but must not crash

---

## 7. Success Criteria for v0.1

1. Successfully transpile `src/main.rs` containing:
   - At least 3 generic functions with trait bounds
   - Struct and enum definitions
   - Pattern matching with Option/Result
   - Vector operations
   - A closure or two
   
2. Output must:
   - Contain zero `&`, `->`, `::`, `<type>`, or `*` (pointer) symbols
   - Be human-readable (follows Iron formatting spec)
   - Parse deterministically (running twice yields identical output)

3. No panics on valid Rust from `cargo new` projects

---

## 8. Future Roadmap (Context Only)

**Phase 2**: Round-trip fidelity (Iron → Rust → Iron equivalence)  
**Phase 3**: Semantic contract validation via embeddings  
**Phase 4**: Fine-tuned LLM trained on Iron↔Rust corpus

---

## 9. Appendix: Example I/O

**Input (Rust):**
```rust
fn find_max<T: PartialOrd>(arr: &[T]) -> Option<&T> {
    let mut max = arr.first()?;
    for item in arr.iter().skip(1) {
        if item > max {
            max = item;
        }
    }
    Some(max)
}
```

**Expected Output (Iron):**
```
function find_max with generic type T implementing PartialOrd
    takes array of type T
    returns optional reference to T
begin
    define mutable max as optional first element of array unwrap or return none
    
    for each item in iterator of array skipping 1 repeat
        if item is greater than max then
            set max equal to item
        end if
    end for
    
    return some of max
end function
```

---

**Document Version**: 0.1  
**Status**: Draft for Implementation  
**Target**: Functional CLI tool for Rust→Iron transpilation
