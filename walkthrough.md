Want a program that...

```
async fn main1() {
    foo
}

async fn main2() {
    bar
}
```

Want that editing 1 fn doesn't require a lot of work

Also want:

- Great error messages
- Lazy compilation
- Nice code

Model:

- Lex the entire file
  - Break it into token trees
    - regular token might be identifier, whitespace
    - `{...}` <-- token tree (big nested object)
- Easily be able to
  - Read the token trees
  - Parse the file "just enough" to get the list of functions or other items
    - but not any more than I need to
- How to represent a token tree
  - not interned, entity has persistent id between executions

in pseudo-code

```
fn source(input_file) { panic!() }

fn lex(input_file: InputFile) -> TokenTree {
    let source = source(input_file);
    let mut tokens = vec![];
    while let Some(token) = next token {
        if token == '{' {
            let token_tree = lex until the `}`;
            tokens.push(BracedBlock(token_tree))
        } else {
            tokens.push(token);
        }
    }
    return TokenTree::new(db, span, tokens)
}
```

parsing part

```
struct Function {
    #[id] name: Word,
    parameters: TokenTree,
    body: TokenTree,
}

#[salsa::tracked(ref)]
fn parse(input_file: InputFile) -> Vec<Item> {
    let token_tree = lex(input_file);
    let mut items = vec![];
    for each token in &token_tree.data(db).tokens {
        if token == fn {
            let name = next token;
            let params = parenthesized list;
            let body = braces list;
            let item = ItemFn::new( // put into database with mapping local to the query
                db,
                name,
                params,
                body,
            );
            items.push(item);
        } else ... {

        } else {
            Diagnostics::push(
                db,
                Diagnostic::new("can't parse item"),
            )
        }
    }
    items
}
```

```
#[salsa::tracked]
fn item_names(db, input_file: InputFile) -> Set<Name> {
    for item in parse(db, input_file) {
        let name = item.name(db);
    }
}
```

```
fn errors(db, input_file: InputFile) {
    item_names::accumulated::<Diagnostics>(db, input_file)
}
```

```
#[salsa::tracked]
fn find_all_references(db, needle: Item, input_file: InputFile) -> Vec<Item> {
    items(db, input_file).iter().filter(|i| item.references(db, needle)).collect()
}

impl Function {
    fn references(&self, needle: Item, db) -> bool {
        let ast = self.validated_tree(db);
        ast.contains(&needle)
    }
}
```

```
fn bar() {

}

fn foo() {
        // Entity Foo
}

fn foo() {
        // Entity Foo1
}
```
