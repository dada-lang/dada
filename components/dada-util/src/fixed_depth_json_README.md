# Debug Field Deny List

This feature allows you to exclude specific fields from the JSON debug output by adding them to a global deny list.

## How It Works

The `DEBUG_FIELD_DENY_LIST` in `fixed_depth_json.rs` is a global list of field names that will be excluded from serialization. When a field with a name in this list is encountered during serialization, it will be skipped entirely, reducing noise in the debug output.

## Usage

To exclude fields from the debug output, simply add their names to the `DEBUG_FIELD_DENY_LIST` in `components/dada-util/src/fixed_depth_json.rs`:

```rust
lazy_static! {
    pub static ref DEBUG_FIELD_DENY_LIST: HashSet<&'static str> = {
        let mut set = HashSet::new();
        
        // Add field names to exclude here
        set.insert("compiler_location");  // Exclude entire location info
        set.insert("file");               // Or just exclude file paths
        set.insert("line");               // Or just exclude line numbers
        set.insert("column");             // Or just exclude column numbers
        
        // Add any other fields you want to exclude
        set.insert("noisy_field_1");
        set.insert("noisy_field_2");
        
        set
    };
}
```

## Common Fields to Exclude

Here are some fields you might want to consider excluding:

1. **Location Information**:
   - `compiler_location` - The entire location struct
   - `file` - Just the file path
   - `line` - Just the line number
   - `column` - Just the column number

2. **Internal Details**:
   - Fields that contain internal implementation details that aren't useful for debugging

3. **Large Data Structures**:
   - Fields that contain large collections or deeply nested structures that make the output hard to read

## Benefits

- **Cleaner Output**: Reduces noise in the debug output, making it easier to focus on relevant information
- **Centralized Configuration**: All excluded fields are defined in one place
- **No Struct Modifications**: You don't need to modify any struct definitions with `#[serde(skip)]` attributes
- **Easy Maintenance**: Simple to add or remove fields from the deny list as needed
