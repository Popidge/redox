//! Iron code emitter
//!
//! This module handles the generation of Iron source code with proper formatting,
//! indentation, and LLM-optimized output structure.

use crate::keywords::sanitize_identifier;

/// Builder for generating Iron code with proper formatting
pub struct IronEmitter {
    output: String,
    indent_level: usize,
    indent_size: usize,
    needs_newline: bool,
}

impl IronEmitter {
    /// Create a new emitter with default settings
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size: 4,
            needs_newline: false,
        }
    }

    /// Create a new emitter with custom indentation size
    pub fn with_indent_size(indent_size: usize) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size,
            needs_newline: false,
        }
    }

    /// Get the current output as a string (for reading without consuming)
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Get the current indentation string
    fn current_indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    /// Write a line with proper indentation
    pub fn write_line(&mut self, content: &str) {
        if self.needs_newline {
            self.output.push('\n');
        }
        self.output.push_str(&self.current_indent());
        self.output.push_str(content);
        self.needs_newline = true;
    }

    /// Write a line without trailing newline
    pub fn write(&mut self, content: &str) {
        self.output.push_str(content);
        self.needs_newline = false;
    }

    /// Write inline content (no indentation)
    pub fn write_inline(&mut self, content: &str) {
        self.output.push_str(content);
    }

    /// Increase indentation level
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Write an empty line
    pub fn write_empty_line(&mut self) {
        if self.needs_newline {
            self.output.push('\n');
        }
        self.output.push('\n');
        self.needs_newline = false;
    }

    /// Start a block with "begin"
    pub fn begin_block(&mut self) {
        self.write_line("begin");
        self.indent();
    }

    /// End a block with "end" and a label
    pub fn end_block(&mut self, label: &str) {
        self.dedent();
        self.write_line(&format!("end {}", label));
    }

    /// Write a comment in Iron format
    pub fn write_comment(&mut self, content: &str) {
        self.write_line(&format!("note that {}", content));
    }

    /// Get the final output (consumes self)
    pub fn finalize(self) -> String {
        self.output
    }

    /// Clone the current output without consuming self
    pub fn clone_output(&self) -> String {
        self.output.clone()
    }

    /// Write a function header
    pub fn write_function_header(
        &mut self,
        name: &str,
        generics: Option<&str>,
        params: &[(String, String)],
        return_type: &str,
    ) {
        let sanitized_name = sanitize_identifier(name);

        if let Some(generic_info) = generics {
            self.write_line(&format!("function {} {}", sanitized_name, generic_info));
        } else {
            self.write_line(&format!("function {}", sanitized_name));
        }

        if !params.is_empty() {
            let param_str = params
                .iter()
                .map(|(name, ty)| format!("{} of {}", name, ty))
                .collect::<Vec<_>>()
                .join(" and ");
            self.write_line(&format!("    takes {}", param_str));
        }

        if return_type != "unit" {
            self.write_line(&format!("    returns {}", return_type));
        }
    }

    /// Write a variable definition
    pub fn write_variable_def(&mut self, name: &str, is_mutable: bool, value: &str) {
        let sanitized_name = sanitize_identifier(name);
        if is_mutable {
            self.write_line(&format!("define mutable {} as {}", sanitized_name, value));
        } else {
            self.write_line(&format!("define {} as {}", sanitized_name, value));
        }
    }

    /// Write a struct definition
    pub fn write_struct_header(&mut self, name: &str, generics: Option<&str>) {
        let sanitized_name = sanitize_identifier(name);
        if let Some(generic_info) = generics {
            self.write_line(&format!(
                "structure {} {} with fields",
                sanitized_name, generic_info
            ));
        } else {
            self.write_line(&format!("structure {} with fields", sanitized_name));
        }
        self.indent();
    }

    /// Write a struct field
    pub fn write_struct_field(&mut self, name: &str, ty: &str) {
        let sanitized_name = sanitize_identifier(name);
        self.write_line(&format!("{} of {}", sanitized_name, ty));
    }

    /// Write enum definition header
    pub fn write_enum_header(&mut self, name: &str, generics: Option<&str>) {
        let sanitized_name = sanitize_identifier(name);
        if let Some(generic_info) = generics {
            self.write_line(&format!(
                "enumeration {} {} with variants",
                sanitized_name, generic_info
            ));
        } else {
            self.write_line(&format!("enumeration {} with variants", sanitized_name));
        }
        self.indent();
    }

    /// Write enum variant (simple)
    pub fn write_enum_variant_simple(&mut self, name: &str) {
        let sanitized_name = sanitize_identifier(name);
        self.write_line(&sanitized_name);
    }

    /// Write enum variant with data
    pub fn write_enum_variant_with_data(&mut self, name: &str, data: &str) {
        let sanitized_name = sanitize_identifier(name);
        self.write_line(&format!("{} of {}", sanitized_name, data));
    }

    /// Write enum variant with named fields
    pub fn write_enum_variant_with_fields(&mut self, name: &str, fields: &[(String, String)]) {
        let sanitized_name = sanitize_identifier(name);
        let field_str = fields
            .iter()
            .map(|(name, ty)| format!("{} of {}", name, ty))
            .collect::<Vec<_>>()
            .join(" and ");
        self.write_line(&format!("{} with {}", sanitized_name, field_str));
    }

    /// Write an if statement header
    pub fn write_if_header(&mut self, condition: &str) {
        self.write_line(&format!("if {} then", condition));
    }

    /// Write an else clause
    pub fn write_else(&mut self) {
        self.write_line("otherwise");
    }

    /// Write end if
    pub fn end_if(&mut self) {
        self.dedent();
        self.write_line("end if");
    }

    /// Write a while loop header
    pub fn write_while_header(&mut self, condition: &str) {
        self.write_line(&format!("while {} repeat", condition));
    }

    /// Write end while
    pub fn end_while(&mut self) {
        self.dedent();
        self.write_line("end while");
    }

    /// Write a for loop header
    pub fn write_for_header(&mut self, var: &str, iterator: &str) {
        let sanitized_var = sanitize_identifier(var);
        self.write_line(&format!(
            "for each {} in {} repeat",
            sanitized_var, iterator
        ));
    }

    /// Write end for
    pub fn end_for(&mut self) {
        self.dedent();
        self.write_line("end for");
    }

    /// Write a match expression header
    pub fn write_match_header(&mut self, expr: &str) {
        self.write_line(&format!("compare {}", expr));
    }

    /// Write a match arm
    pub fn write_match_arm(&mut self, pattern: &str, body: &str) {
        self.write_line(&format!("    case {} then {}", pattern, body));
    }

    /// Write end match
    pub fn end_match(&mut self) {
        self.write_line("end compare");
    }

    /// Write a return statement
    pub fn write_return(&mut self, value: Option<&str>) {
        if let Some(val) = value {
            self.write_line(&format!("return {}", val));
        } else {
            self.write_line("return");
        }
    }

    /// Write a verbatim Rust item payload
    pub fn write_verbatim_item(&mut self, rust_item: &str) {
        self.write_line(&format!("verbatim item \"{}\"", rust_item.escape_default()));
    }

    /// Write an assignment
    pub fn write_assignment(&mut self, target: &str, value: &str) {
        self.write_line(&format!("set {} equal to {}", target, value));
    }
}

impl Default for IronEmitter {
    fn default() -> Self {
        Self::new()
    }
}
