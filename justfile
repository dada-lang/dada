test:
    cargo test --all --workspace --all-targets

# Generate documentation with private items included (recommended)
doc:
    cargo doc --workspace --no-deps --document-private-items

# Generate and open documentation 
doc-open:
    cargo doc --workspace --no-deps --document-private-items --open

# Generate documentation and serve it locally on http://localhost:8000
doc-serve:
    cargo doc --workspace --no-deps --document-private-items
    @echo "Documentation generated. Starting server at http://localhost:8000"
    @echo "Visit http://localhost:8000/dada/ to view the main documentation"
    cd target/doc && python3 -m http.server 8000