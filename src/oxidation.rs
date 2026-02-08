//! Oxidation - Iron to Rust transpiler
//!
//! Converts Iron AST into valid Rust source code.

use crate::iron_ast::*;

pub struct Oxidizer {
    output: String,
    indent_level: usize,
}

impl Oxidizer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn oxidize(&mut self, file: &IronFile) -> String {
        for (i, item) in file.items.iter().enumerate() {
            if i > 0 {
                self.output.push_str("\n\n");
            }
            self.oxidize_item(item);
        }
        self.output.clone()
    }

    fn oxidize_item(&mut self, item: &IronItem) {
        match item {
            IronItem::Function(func) => self.oxidize_function(func),
            IronItem::Struct(strct) => self.oxidize_struct(strct),
            IronItem::Enum(enm) => self.oxidize_enum(enm),
            IronItem::Static(stat) => self.oxidize_static(stat),
            IronItem::Const(cnst) => self.oxidize_const(cnst),
            IronItem::TypeAlias(alias) => self.oxidize_type_alias(alias),
            IronItem::Verbatim(item) => self.oxidize_verbatim_item(item),
        }
    }

    fn oxidize_function(&mut self, func: &IronFunction) {
        // Function signature
        self.output.push_str("fn ");
        self.output.push_str(&func.name);

        // Generics
        if !func.generics.is_empty() {
            self.output.push_str("<");
            for (i, generic) in func.generics.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&generic.name);
                if !generic.bounds.is_empty() {
                    self.output.push_str(": ");
                    for (j, bound) in generic.bounds.iter().enumerate() {
                        if j > 0 {
                            self.output.push_str(" + ");
                        }
                        self.output.push_str(&bound.trait_name);
                    }
                }
            }
            self.output.push_str(">");
        }

        // Parameters
        self.output.push_str("(");
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&param.name);
            self.output.push_str(": ");
            self.oxidize_type(&param.ty);
        }
        self.output.push_str(")");

        // Return type
        if let Some(ret) = &func.return_type {
            self.output.push_str(" -> ");
            self.oxidize_type(ret);
        }

        // Body
        self.output.push_str(" {\n");
        self.indent_level += 1;
        let body_len = func.body.len();
        for (i, stmt) in func.body.iter().enumerate() {
            let is_last = i == body_len - 1;
            self.oxidize_statement(stmt, is_last);
        }
        self.indent_level -= 1;
        self.output.push_str("}\n");
    }

    fn oxidize_struct(&mut self, strct: &IronStruct) {
        self.output.push_str("struct ");
        self.output.push_str(&strct.name);

        // Generics
        if !strct.generics.is_empty() {
            self.output.push_str("<");
            for (i, generic) in strct.generics.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&generic.name);
            }
            self.output.push_str(">");
        }

        // Fields
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for field in &strct.fields {
            self.write_indent();
            self.output.push_str(&field.name);
            self.output.push_str(": ");
            self.oxidize_type(&field.ty);
            self.output.push_str(",\n");
        }
        self.indent_level -= 1;
        self.output.push_str("}\n");
    }

    fn oxidize_enum(&mut self, enm: &IronEnum) {
        self.output.push_str("enum ");
        self.output.push_str(&enm.name);

        // Generics
        if !enm.generics.is_empty() {
            self.output.push_str("<");
            for (i, generic) in enm.generics.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&generic.name);
            }
            self.output.push_str(">");
        }

        // Variants
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for variant in &enm.variants {
            self.write_indent();
            self.output.push_str(&variant.name);

            if let Some(data) = &variant.data {
                match data {
                    IronVariantData::Type(ty) => {
                        self.output.push_str("(");
                        self.oxidize_type(ty);
                        self.output.push_str(")");
                    }
                    IronVariantData::Fields(fields) => {
                        self.output.push_str(" {");
                        for (i, field) in fields.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.output.push_str(&field.name);
                            self.output.push_str(": ");
                            self.oxidize_type(&field.ty);
                        }
                        self.output.push_str("}");
                    }
                }
            }
            self.output.push_str(",\n");
        }
        self.indent_level -= 1;
        self.output.push_str("}\n");
    }

    fn oxidize_static(&mut self, stat: &IronStatic) {
        self.output.push_str("static ");
        if stat.mutable {
            self.output.push_str("mut ");
        }
        self.output.push_str(&stat.name);
        self.output.push_str(": ");
        self.oxidize_type(&stat.ty);
        self.output.push_str(" = ");
        self.oxidize_expr(&stat.value);
        self.output.push_str(";\n");
    }

    fn oxidize_const(&mut self, cnst: &IronConst) {
        self.output.push_str("const ");
        self.output.push_str(&cnst.name);
        self.output.push_str(": ");
        self.oxidize_type(&cnst.ty);
        self.output.push_str(" = ");
        self.oxidize_expr(&cnst.value);
        self.output.push_str(";\n");
    }

    fn oxidize_type_alias(&mut self, alias: &IronTypeAlias) {
        self.output.push_str("type ");
        self.output.push_str(&alias.name);

        if !alias.generics.is_empty() {
            self.output.push_str("<");
            for (i, generic) in alias.generics.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&generic.name);
                if !generic.bounds.is_empty() {
                    self.output.push_str(": ");
                    for (j, bound) in generic.bounds.iter().enumerate() {
                        if j > 0 {
                            self.output.push_str(" + ");
                        }
                        self.output.push_str(&bound.trait_name);
                    }
                }
            }
            self.output.push_str(">");
        }

        self.output.push_str(" = ");
        self.oxidize_type(&alias.ty);
        self.output.push_str(";\n");
    }

    fn oxidize_verbatim_item(&mut self, item: &str) {
        self.output.push_str(item);
        self.output.push_str("\n");
    }

    fn oxidize_type(&mut self, ty: &IronType) {
        match ty {
            IronType::Named(name) => {
                // Map Iron type names back to Rust
                let rust_name = match name.as_str() {
                    "boolean" => "bool".to_string(),
                    "character" => "char".to_string(),
                    "string" => "String".to_string(),
                    "string slice" => "str".to_string(),
                    "list" => "Vec".to_string(),
                    "optional" => "Option".to_string(),
                    "result" => "Result".to_string(),
                    "hash map" => "HashMap".to_string(),
                    "box" => "Box".to_string(),
                    "reference counted" => "Rc".to_string(),
                    "atomic reference counted" => "Arc".to_string(),
                    "unit" => "()".to_string(),
                    "error" => "dyn std::error::Error".to_string(),
                    "std::error::Error" => "dyn std::error::Error".to_string(),
                    "std::fmt::Display" => "dyn std::fmt::Display".to_string(),
                    _ => name.to_string(),
                };
                self.output.push_str(&rust_name);
            }
            IronType::Reference(inner) => {
                self.output.push_str("&");
                self.oxidize_type(inner);
            }
            IronType::MutableReference(inner) => {
                self.output.push_str("&mut ");
                self.oxidize_type(inner);
            }
            IronType::RawPointer(inner) => {
                self.output.push_str("*const ");
                self.oxidize_type(inner);
            }
            IronType::MutableRawPointer(inner) => {
                self.output.push_str("*mut ");
                self.oxidize_type(inner);
            }
            IronType::Optional(inner) => {
                self.output.push_str("std::option::Option<");
                self.oxidize_type(inner);
                self.output.push_str(">");
            }
            IronType::Result(ok, err) => {
                self.output.push_str("std::result::Result<");
                self.oxidize_type(ok);
                self.output.push_str(", ");
                self.oxidize_type(err);
                self.output.push_str(">");
            }
            IronType::List(inner) => {
                self.output.push_str("Vec<");
                self.oxidize_type(inner);
                self.output.push_str(">");
            }
            IronType::BoxType(inner) => {
                self.output.push_str("Box<");
                self.oxidize_type(inner);
                self.output.push_str(">");
            }
            IronType::Tuple(types) => {
                self.output.push_str("(");
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_type(ty);
                }
                self.output.push_str(")");
            }
            IronType::Array(inner) => {
                self.output.push_str("[");
                self.oxidize_type(inner);
                self.output.push_str("]");
            }
            IronType::Slice(inner) => {
                // Slice is just [T], the reference is handled by Reference/MutableReference
                self.output.push_str("[");
                self.oxidize_type(inner);
                self.output.push_str("]");
            }
            IronType::Function(params, ret) => {
                self.output.push_str("fn(");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_type(param);
                }
                self.output.push_str(")");
                self.output.push_str(" -> ");
                self.oxidize_type(ret);
            }
            IronType::Generic(name, _bounds) => {
                self.output.push_str(name);
            }
        }
    }

    fn oxidize_statement(&mut self, stmt: &IronStmt, is_last: bool) {
        self.write_indent();

        match stmt {
            IronStmt::Let {
                name,
                mutable,
                value,
            } => {
                self.output.push_str("let ");
                if *mutable {
                    self.output.push_str("mut ");
                }
                self.output.push_str(name);
                self.output.push_str(" = ");
                self.oxidize_expr(value);
                self.output.push_str(";\n");
            }
            IronStmt::Assign { target, value } => {
                self.oxidize_expr(target);
                self.output.push_str(" = ");
                self.oxidize_expr(value);
                self.output.push_str(";\n");
            }
            IronStmt::Expr(expr) => {
                self.oxidize_expr(expr);
                if is_last {
                    // Tail expression - no semicolon
                    self.output.push_str("\n");
                } else {
                    self.output.push_str(";\n");
                }
            }
            IronStmt::Return(expr) => {
                self.output.push_str("return");
                if let Some(val) = expr {
                    self.output.push_str(" ");
                    self.oxidize_expr(val);
                }
                self.output.push_str(";\n");
            }
            IronStmt::Break => {
                self.output.push_str("break;\n");
            }
            IronStmt::Continue => {
                self.output.push_str("continue;\n");
            }
            IronStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.output.push_str("if ");
                self.oxidize_expr(condition);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                let then_len = then_block.len();
                for (i, s) in then_block.iter().enumerate() {
                    self.oxidize_statement(s, i == then_len - 1);
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}");

                if let Some(else_blk) = else_block {
                    self.output.push_str(" else {\n");
                    self.indent_level += 1;
                    let else_len = else_blk.len();
                    for (i, s) in else_blk.iter().enumerate() {
                        self.oxidize_statement(s, i == else_len - 1);
                    }
                    self.indent_level -= 1;
                    self.write_indent();
                    self.output.push_str("}");
                }
                self.output.push_str("\n");
            }
            IronStmt::While { condition, body } => {
                self.output.push_str("while ");
                self.oxidize_expr(condition);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                let body_len = body.len();
                for (i, s) in body.iter().enumerate() {
                    self.oxidize_statement(s, i == body_len - 1);
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
            IronStmt::For {
                var,
                iterator,
                body,
            } => {
                self.output.push_str("for ");
                self.output.push_str(var);
                self.output.push_str(" in ");
                self.oxidize_expr(iterator);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                let body_len = body.len();
                for (i, s) in body.iter().enumerate() {
                    self.oxidize_statement(s, i == body_len - 1);
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
            IronStmt::Match { expr, arms } => {
                self.output.push_str("match ");
                self.oxidize_expr(expr);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for (pattern, arm_expr) in arms {
                    self.write_indent();
                    self.oxidize_pattern(pattern);
                    self.output.push_str(" => ");
                    self.oxidize_expr(arm_expr);
                    self.output.push_str(",\n");
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
        }
    }

    fn oxidize_expr(&mut self, expr: &IronExpr) {
        match expr {
            IronExpr::Identifier(name) => {
                self.output.push_str(name);
            }
            IronExpr::String(s) => {
                self.output.push_str("\"");
                self.output.push_str(s);
                self.output.push_str("\"");
            }
            IronExpr::Integer(n) => {
                self.output.push_str(n);
            }
            IronExpr::Float(n) => {
                self.output.push_str(n);
            }
            IronExpr::Boolean(b) => {
                self.output.push_str(if *b { "true" } else { "false" });
            }
            IronExpr::Binary { left, op, right } => {
                self.oxidize_expr(left);
                self.output.push_str(" ");
                self.oxidize_binary_op(op);
                self.output.push_str(" ");
                self.oxidize_expr(right);
            }
            IronExpr::Unary { op, expr } => {
                self.oxidize_unary_op(op);
                self.output.push_str(" ");
                self.oxidize_expr(expr);
            }
            IronExpr::Call { func, args } => {
                self.oxidize_expr(func);
                self.output.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_expr(arg);
                }
                self.output.push_str(")");
            }
            IronExpr::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.oxidize_expr(receiver);
                self.output.push_str(".");
                self.output.push_str(method);
                self.output.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_expr(arg);
                }
                self.output.push_str(")");
            }
            IronExpr::AssociatedFunctionCall {
                type_name,
                function,
                args,
            } => {
                self.output.push_str(type_name);
                self.output.push_str("::");
                self.output.push_str(function);
                self.output.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_expr(arg);
                }
                self.output.push_str(")");
            }
            IronExpr::Macro {
                name,
                args,
                bracket,
            } => {
                self.output.push_str(name);
                if *bracket {
                    self.output.push_str("![");
                    if !args.is_empty() {
                        self.output.push_str(args);
                    }
                    self.output.push_str("]");
                } else {
                    self.output.push_str("!(");
                    if !args.is_empty() {
                        self.output.push_str(args);
                    }
                    self.output.push_str(")");
                }
            }
            IronExpr::FieldAccess { base, field } => {
                self.oxidize_expr(base);
                self.output.push_str(".");
                self.output.push_str(field);
            }
            IronExpr::Try { expr } => {
                self.oxidize_expr(expr);
                self.output.push_str("?");
            }
            IronExpr::Some(expr) => {
                self.output.push_str("Some(");
                self.oxidize_expr(expr);
                self.output.push_str(")");
            }
            IronExpr::None => {
                self.output.push_str("None");
            }
            IronExpr::Ok(expr) => {
                self.output.push_str("Ok(");
                self.oxidize_expr(expr);
                self.output.push_str(")");
            }
            IronExpr::Err(expr) => {
                self.output.push_str("Err(");
                self.oxidize_expr(expr);
                self.output.push_str(")");
            }
            IronExpr::Tuple(elems) => {
                self.output.push_str("(");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_expr(elem);
                }
                self.output.push_str(")");
            }
            IronExpr::Array(elems) => {
                self.output.push_str("[");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_expr(elem);
                }
                self.output.push_str("]");
            }
            IronExpr::Struct { name, fields } => {
                self.output.push_str(name);
                self.output.push_str(" {");
                for (i, (field, expr)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&field.name);
                    self.output.push_str(": ");
                    self.oxidize_expr(expr);
                }
                self.output.push_str("}");
            }
            IronExpr::Index { base, index } => {
                self.oxidize_expr(base);
                self.output.push_str("[");
                self.oxidize_expr(index);
                self.output.push_str("]");
            }
            IronExpr::Range {
                start,
                end,
                inclusive,
            } => {
                if let Some(s) = start {
                    self.oxidize_expr(s);
                }
                if *inclusive {
                    self.output.push_str("..=");
                } else {
                    self.output.push_str("..");
                }
                if let Some(e) = end {
                    self.oxidize_expr(e);
                }
            }
            IronExpr::Closure { params, body } => {
                self.output.push_str("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&param.name);
                }
                self.output.push_str("| {\n");
                self.indent_level += 1;
                let body_len = body.len();
                for (i, stmt) in body.iter().enumerate() {
                    self.oxidize_statement(stmt, i == body_len - 1);
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}");
            }
        }
    }

    fn oxidize_binary_op(&mut self, op: &IronBinaryOp) {
        let op_str = match op {
            IronBinaryOp::Add => "+",
            IronBinaryOp::Sub => "-",
            IronBinaryOp::Mul => "*",
            IronBinaryOp::Div => "/",
            IronBinaryOp::Mod => "%",
            IronBinaryOp::And => "&&",
            IronBinaryOp::Or => "||",
            IronBinaryOp::Eq => "==",
            IronBinaryOp::Ne => "!=",
            IronBinaryOp::Lt => "<",
            IronBinaryOp::Le => "<=",
            IronBinaryOp::Gt => ">",
            IronBinaryOp::Ge => ">=",
            IronBinaryOp::BitAnd => "&",
            IronBinaryOp::BitOr => "|",
            IronBinaryOp::BitXor => "^",
            IronBinaryOp::Shl => "<<",
            IronBinaryOp::Shr => ">>",
        };
        self.output.push_str(op_str);
    }

    fn oxidize_unary_op(&mut self, op: &IronUnaryOp) {
        let op_str = match op {
            IronUnaryOp::Not => "!",
            IronUnaryOp::Neg => "-",
            IronUnaryOp::Deref => "*",
        };
        self.output.push_str(op_str);
    }

    fn oxidize_pattern(&mut self, pattern: &IronPattern) {
        match pattern {
            IronPattern::Identifier(name) => {
                self.output.push_str(name);
            }
            IronPattern::Wildcard => {
                self.output.push_str("_");
            }
            IronPattern::Literal(expr) => {
                self.oxidize_expr(expr);
            }
            IronPattern::Tuple(patterns) => {
                self.output.push_str("(");
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.oxidize_pattern(pat);
                }
                self.output.push_str(")");
            }
            IronPattern::Struct { name, fields } => {
                self.output.push_str(name);
                self.output.push_str(" {");
                for (i, (field, pat)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&field.name);
                    self.output.push_str(": ");
                    self.oxidize_pattern(pat);
                }
                self.output.push_str("}");
            }
            IronPattern::Variant {
                enum_name,
                variant_name,
                data,
            } => {
                self.output.push_str(enum_name);
                self.output.push_str("::");
                self.output.push_str(variant_name);
                if let Some(d) = data {
                    self.output.push_str("(");
                    self.oxidize_pattern(d);
                    self.output.push_str(")");
                }
            }
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}

impl Default for Oxidizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iron_parser::IronParser;

    #[test]
    fn test_oxidize_simple_function() {
        let iron_input = r#"function hello
begin
    return 42
end function"#;

        let mut parser = IronParser::new(iron_input);
        let ast = parser.parse().unwrap();

        let mut oxidizer = Oxidizer::new();
        let rust = oxidizer.oxidize(&ast);

        assert!(rust.contains("fn hello()"));
        assert!(rust.contains("return 42"));
    }
}
