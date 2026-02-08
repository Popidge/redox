//! Rust AST parser using syn::Visit trait
//!
//! This module implements the visitor pattern to traverse Rust syntax trees
//! and convert them to Iron code using the emitter.

use crate::emitter::IronEmitter;
use crate::keywords::sanitize_identifier;
use crate::mappings::{map_binary_op, map_fn_arg, map_return_type, map_type_to_iron, map_unary_op};
use quote::ToTokens;
use syn::visit::Visit;
use syn::{Attribute, Expr, File, GenericParam, Item, Member, Pat, Stmt};

/// Parser that visits Rust AST and emits Iron code
pub struct IronParser {
    emitter: IronEmitter,
    errors: Vec<String>,
}

impl IronParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            emitter: IronEmitter::new(),
            errors: Vec::new(),
        }
    }

    /// Parse a Rust file and return Iron code
    pub fn parse_file(&mut self, file: &File) -> Result<String, Vec<String>> {
        self.visit_file(file);

        if self.errors.is_empty() {
            // Clone the emitter output without consuming it
            Ok(self.emitter.clone_output())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Process attributes (comments and doc comments)
    fn process_attributes(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let Ok(syn::Meta::NameValue(meta)) = attr.parse_args::<syn::Meta>() {
                    if let syn::Expr::Lit(expr_lit) = meta.value {
                        if let syn::Lit::Str(lit_str) = expr_lit.lit {
                            self.emitter.write_comment(&lit_str.value());
                        }
                    }
                }
            }
        }
    }

    /// Format a type parameter bound (trait bound) to Iron
    fn format_type_param_bound(bound: &syn::TypeParamBound) -> String {
        match bound {
            syn::TypeParamBound::Trait(trait_bound) => {
                let path = &trait_bound.path;
                path.segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            syn::TypeParamBound::Lifetime(lt) => {
                format!("lifetime {}", lt.ident)
            }
            _ => "unknown bound".to_string(),
        }
    }

    fn emit_verbatim_item(&mut self, item: &Item) {
        let rust_item = item.to_token_stream().to_string();
        self.emitter.write_verbatim_item(&rust_item);
        self.emitter.write_empty_line();
    }

    fn type_contains_impl_trait(ty: &syn::Type) -> bool {
        match ty {
            syn::Type::ImplTrait(_) => true,
            syn::Type::Reference(type_ref) => Self::type_contains_impl_trait(&type_ref.elem),
            syn::Type::Ptr(type_ptr) => Self::type_contains_impl_trait(&type_ptr.elem),
            syn::Type::Tuple(tuple) => tuple.elems.iter().any(Self::type_contains_impl_trait),
            syn::Type::Array(array) => Self::type_contains_impl_trait(&array.elem),
            syn::Type::Slice(slice) => Self::type_contains_impl_trait(&slice.elem),
            syn::Type::Paren(paren) => Self::type_contains_impl_trait(&paren.elem),
            syn::Type::Path(type_path) => type_path.path.segments.iter().any(|segment| {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    args.args.iter().any(|arg| match arg {
                        syn::GenericArgument::Type(t) => Self::type_contains_impl_trait(t),
                        _ => false,
                    })
                } else {
                    false
                }
            }),
            _ => false,
        }
    }

    fn fn_signature_contains_impl_trait(sig: &syn::Signature) -> bool {
        for input in &sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input
                && Self::type_contains_impl_trait(&pat_type.ty)
            {
                return true;
            }
        }

        if let syn::ReturnType::Type(_, ty) = &sig.output
            && Self::type_contains_impl_trait(ty)
        {
            return true;
        }

        false
    }

    fn fn_body_needs_verbatim(item_fn: &syn::ItemFn) -> bool {
        let body_tokens = item_fn.block.to_token_stream().to_string();
        body_tokens.contains('?')
            || body_tokens.contains("::<")
            || body_tokens.contains("if let")
            || body_tokens.contains("while let")
            || body_tokens.contains("match ")
    }
}

impl<'ast> Visit<'ast> for IronParser {
    fn visit_file(&mut self, file: &'ast File) {
        for item in &file.items {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Fn(item_fn) => {
                if Self::fn_signature_contains_impl_trait(&item_fn.sig)
                    || Self::fn_body_needs_verbatim(item_fn)
                {
                    self.emit_verbatim_item(item);
                    return;
                }

                self.process_attributes(&item_fn.attrs);

                // Process generics
                let generics_str = if item_fn.sig.generics.params.is_empty() {
                    None
                } else {
                    let gen_params: Vec<String> = item_fn
                        .sig
                        .generics
                        .params
                        .iter()
                        .map(|p| match p {
                            GenericParam::Type(type_param) => {
                                let name = type_param.ident.to_string();
                                if type_param.bounds.is_empty() {
                                    format!("with generic type {}", sanitize_identifier(&name))
                                } else {
                                    let bounds: Vec<String> = type_param
                                        .bounds
                                        .iter()
                                        .map(|b| Self::format_type_param_bound(b))
                                        .collect();
                                    format!(
                                        "with generic type {} implementing {}",
                                        sanitize_identifier(&name),
                                        bounds.join(" and ")
                                    )
                                }
                            }
                            GenericParam::Lifetime(lt) => {
                                format!("with lifetime {}", lt.lifetime.ident)
                            }
                            GenericParam::Const(const_param) => {
                                format!("with const generic {}", const_param.ident)
                            }
                        })
                        .collect();
                    Some(gen_params.join(" "))
                };

                // Process parameters
                let params: Vec<(String, String)> =
                    item_fn.sig.inputs.iter().filter_map(map_fn_arg).collect();

                // Process return type
                let return_type = map_return_type(&item_fn.sig.output);

                // Get function name
                let fn_name = item_fn.sig.ident.to_string();

                // Emit function header
                self.emitter.write_function_header(
                    &fn_name,
                    generics_str.as_deref(),
                    &params,
                    &return_type,
                );

                // Emit function body
                self.emitter.begin_block();
                for stmt in &item_fn.block.stmts {
                    self.visit_stmt(stmt);
                }
                self.emitter.end_block("function");
                self.emitter.write_empty_line();
            }

            Item::Struct(item_struct) => {
                self.process_attributes(&item_struct.attrs);

                let name = item_struct.ident.to_string();

                // Process generics
                let generics_str = if item_struct.generics.params.is_empty() {
                    None
                } else {
                    let gen_params: Vec<String> = item_struct
                        .generics
                        .params
                        .iter()
                        .map(|p| match p {
                            GenericParam::Type(type_param) => {
                                format!("with generic type {}", type_param.ident)
                            }
                            _ => "".to_string(),
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                    if gen_params.is_empty() {
                        None
                    } else {
                        Some(gen_params.join(" "))
                    }
                };

                self.emitter
                    .write_struct_header(&name, generics_str.as_deref());

                // Process fields
                match &item_struct.fields {
                    syn::Fields::Named(fields_named) => {
                        for field in &fields_named.named {
                            if let Some(ident) = &field.ident {
                                let field_name = ident.to_string();
                                let field_type = map_type_to_iron(&field.ty);
                                self.emitter.write_struct_field(&field_name, &field_type);
                            }
                        }
                    }
                    syn::Fields::Unnamed(fields_unnamed) => {
                        for (idx, field) in fields_unnamed.unnamed.iter().enumerate() {
                            let field_type = map_type_to_iron(&field.ty);
                            self.emitter
                                .write_struct_field(&format!("field{}", idx), &field_type);
                        }
                    }
                    syn::Fields::Unit => {
                        // Unit struct - no fields
                    }
                }

                self.emitter.dedent();
                self.emitter.write_line("end structure");
                self.emitter.write_empty_line();
            }

            Item::Enum(item_enum) => {
                self.process_attributes(&item_enum.attrs);

                let name = item_enum.ident.to_string();

                // Process generics
                let generics_str = if item_enum.generics.params.is_empty() {
                    None
                } else {
                    let gen_params: Vec<String> = item_enum
                        .generics
                        .params
                        .iter()
                        .map(|p| match p {
                            GenericParam::Type(type_param) => {
                                format!("with generic type {}", type_param.ident)
                            }
                            _ => "".to_string(),
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                    if gen_params.is_empty() {
                        None
                    } else {
                        Some(gen_params.join(" "))
                    }
                };

                self.emitter
                    .write_enum_header(&name, generics_str.as_deref());

                // Process variants
                for variant in &item_enum.variants {
                    let variant_name = variant.ident.to_string();

                    match &variant.fields {
                        syn::Fields::Unit => {
                            self.emitter.write_enum_variant_simple(&variant_name);
                        }
                        syn::Fields::Unnamed(fields_unnamed) => {
                            if fields_unnamed.unnamed.len() == 1 {
                                let ty = map_type_to_iron(&fields_unnamed.unnamed[0].ty);
                                self.emitter
                                    .write_enum_variant_with_data(&variant_name, &ty);
                            } else {
                                let types: Vec<String> = fields_unnamed
                                    .unnamed
                                    .iter()
                                    .map(|f| map_type_to_iron(&f.ty))
                                    .collect();
                                self.emitter.write_enum_variant_with_data(
                                    &variant_name,
                                    &format!("tuple of {}", types.join(" and ")),
                                );
                            }
                        }
                        syn::Fields::Named(fields_named) => {
                            let fields: Vec<(String, String)> = fields_named
                                .named
                                .iter()
                                .filter_map(|f| {
                                    f.ident
                                        .as_ref()
                                        .map(|ident| (ident.to_string(), map_type_to_iron(&f.ty)))
                                })
                                .collect();
                            self.emitter
                                .write_enum_variant_with_fields(&variant_name, &fields);
                        }
                    }
                }

                self.emitter.dedent();
                self.emitter.write_line("end enumeration");
                self.emitter.write_empty_line();
            }

            Item::Static(item_static) => {
                self.process_attributes(&item_static.attrs);
                let name = item_static.ident.to_string();
                let ty = map_type_to_iron(&item_static.ty);

                // Check mutability - StaticMutability is not an Option, it's an enum
                let is_mut = matches!(&item_static.mutability, syn::StaticMutability::Mut(_));

                if is_mut {
                    self.emitter
                        .write_line(&format!("static mutable {} of {}", name, ty));
                } else {
                    self.emitter
                        .write_line(&format!("static {} of {}", name, ty));
                }

                self.emitter.begin_block();
                self.visit_expr(&item_static.expr);
                self.emitter.end_block("static");
                self.emitter.write_empty_line();
            }

            Item::Const(item_const) => {
                self.process_attributes(&item_const.attrs);
                let name = item_const.ident.to_string();
                let ty = map_type_to_iron(&item_const.ty);

                self.emitter
                    .write_line(&format!("constant {} of {}", name, ty));

                self.emitter.begin_block();
                self.visit_expr(&item_const.expr);
                self.emitter.end_block("constant");
                self.emitter.write_empty_line();
            }

            Item::Type(item_type) => {
                self.process_attributes(&item_type.attrs);
                let name = item_type.ident.to_string();
                let ty = map_type_to_iron(&item_type.ty);

                let generics_str = if item_type.generics.params.is_empty() {
                    None
                } else {
                    let gen_params: Vec<String> = item_type
                        .generics
                        .params
                        .iter()
                        .map(|p| match p {
                            GenericParam::Type(type_param) => {
                                if type_param.bounds.is_empty() {
                                    format!("with generic type {}", type_param.ident)
                                } else {
                                    let bounds: Vec<String> = type_param
                                        .bounds
                                        .iter()
                                        .map(|b| Self::format_type_param_bound(b))
                                        .collect();
                                    format!(
                                        "with generic type {} implementing {}",
                                        type_param.ident,
                                        bounds.join(" and ")
                                    )
                                }
                            }
                            _ => "".to_string(),
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                    if gen_params.is_empty() {
                        None
                    } else {
                        Some(gen_params.join(" "))
                    }
                };

                if let Some(generics_str) = generics_str {
                    self.emitter.write_line(&format!(
                        "type {} {} as {}",
                        sanitize_identifier(&name),
                        generics_str,
                        ty
                    ));
                } else {
                    self.emitter.write_line(&format!(
                        "type {} as {}",
                        sanitize_identifier(&name),
                        ty
                    ));
                }
                self.emitter.write_empty_line();
            }

            _ => {
                // Preserve unsupported items as verbatim Rust to avoid fidelity loss.
                self.emit_verbatim_item(item);
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        match stmt {
            Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    let var_name = match &local.pat {
                        Pat::Ident(pat_ident) => {
                            let name = pat_ident.ident.to_string();
                            let is_mut = pat_ident.mutability.is_some();

                            let value_str = self.expr_to_string(&init.expr);
                            self.emitter.write_variable_def(&name, is_mut, &value_str);
                            return;
                        }
                        Pat::Type(pat_type) => {
                            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                                let name = pat_ident.ident.to_string();
                                let is_mut = pat_ident.mutability.is_some();

                                let value_str = self.expr_to_string(&init.expr);
                                self.emitter.write_variable_def(&name, is_mut, &value_str);
                                return;
                            }
                            "unknown".to_string()
                        }
                        _ => "unknown".to_string(),
                    };

                    let value_str = self.expr_to_string(&init.expr);
                    self.emitter
                        .write_variable_def(&var_name, false, &value_str);
                }
            }

            Stmt::Item(item) => {
                self.visit_item(item);
            }

            Stmt::Expr(expr, _semi) => {
                // Check if this is a control flow expression that needs special handling
                match expr {
                    Expr::ForLoop(for_loop) => {
                        self.emit_for_loop(for_loop);
                    }
                    Expr::While(while_loop) => {
                        self.emit_while_loop(while_loop);
                    }
                    Expr::If(if_expr) => {
                        self.emit_if_statement(if_expr);
                    }
                    _ => {
                        let expr_str = self.expr_to_string(expr);
                        if !expr_str.is_empty() {
                            self.emitter.write_line(&expr_str);
                        }
                    }
                }
            }

            Stmt::Macro(_stmt_macro) => {
                // Macros are not expanded in v0.1
                self.emitter.write_line("macro definition not expanded");
            }
        }
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        let expr_str = self.expr_to_string(expr);
        if !expr_str.is_empty() {
            self.emitter.write_line(&expr_str);
        }
    }
}

impl IronParser {
    /// Convert an expression to its Iron string representation
    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                syn::Lit::Str(s) => format!("\"{}\"", s.value()),
                syn::Lit::ByteStr(_) => "byte string".to_string(),
                syn::Lit::Byte(_) => "byte literal".to_string(),
                syn::Lit::Char(c) => format!("'{}'", c.value()),
                syn::Lit::Int(i) => i.base10_digits().to_string(),
                syn::Lit::Float(f) => f.base10_digits().to_string(),
                syn::Lit::Bool(b) => b.value.to_string(),
                syn::Lit::Verbatim(_) => "verbatim".to_string(),
                _ => "unknown literal".to_string(),
            },

            Expr::Path(expr_path) => {
                if let Some(ident) = expr_path.path.get_ident() {
                    sanitize_identifier(&ident.to_string())
                } else {
                    expr_path
                        .path
                        .segments
                        .iter()
                        .map(|s| sanitize_identifier(&s.ident.to_string()))
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            }

            Expr::Binary(expr_binary) => {
                let left = self.expr_to_string(&expr_binary.left);
                let op = map_binary_op(&expr_binary.op);
                let right = self.expr_to_string(&expr_binary.right);
                format!("{} {} {}", left, op, right)
            }

            Expr::Unary(expr_unary) => {
                let op = map_unary_op(&expr_unary.op);
                let operand = self.expr_to_string(&expr_unary.expr);
                format!("{} {}", op, operand)
            }

            Expr::Call(expr_call) => {
                // Check if this is an associated function call like T::default()
                if let Expr::Path(func_path) = &*expr_call.func {
                    let segments: Vec<_> = func_path.path.segments.iter().collect();
                    if segments.len() >= 2 {
                        // Associated function: T::method() or Type::method()
                        let type_name = segments[..segments.len() - 1]
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::");
                        let method_name = &segments.last().unwrap().ident.to_string();
                        let args: Vec<String> = expr_call
                            .args
                            .iter()
                            .map(|arg| self.expr_to_string(arg))
                            .collect();

                        if args.is_empty() {
                            return format!(
                                "call associated function {} on {}",
                                sanitize_identifier(method_name),
                                sanitize_identifier(&type_name)
                            );
                        } else {
                            return format!(
                                "call associated function {} on {} with {}",
                                sanitize_identifier(method_name),
                                sanitize_identifier(&type_name),
                                args.join(" and ")
                            );
                        }
                    }
                }

                let func = self.expr_to_string(&expr_call.func);
                let args: Vec<String> = expr_call
                    .args
                    .iter()
                    .map(|arg| self.expr_to_string(arg))
                    .collect();

                if func == "Some" {
                    if let Some(arg) = args.first() {
                        format!("some of {}", arg)
                    } else {
                        "some".to_string()
                    }
                } else if func == "None" {
                    "none".to_string()
                } else if func == "Ok" {
                    if let Some(arg) = args.first() {
                        format!("ok of {}", arg)
                    } else {
                        "ok".to_string()
                    }
                } else if func == "Err" {
                    if let Some(arg) = args.first() {
                        format!("error of {}", arg)
                    } else {
                        "error".to_string()
                    }
                } else {
                    format!("call {} with {}", func, args.join(" and "))
                }
            }

            Expr::MethodCall(expr_method) => {
                let receiver = self.expr_to_string(&expr_method.receiver);
                let method = sanitize_identifier(&expr_method.method.to_string());
                let args: Vec<String> = expr_method
                    .args
                    .iter()
                    .map(|arg| self.expr_to_string(arg))
                    .collect();

                if args.is_empty() {
                    format!("call method {} on {}", method, receiver)
                } else {
                    format!(
                        "call method {} on {} with {}",
                        method,
                        receiver,
                        args.join(" and ")
                    )
                }
            }

            Expr::Field(expr_field) => {
                let base = self.expr_to_string(&expr_field.base);
                let field_name = match &expr_field.member {
                    Member::Named(ident) => sanitize_identifier(&ident.to_string()),
                    Member::Unnamed(idx) => format!("field{}", idx.index),
                };
                format!("field {} of {}", field_name, base)
            }

            Expr::If(_expr_if) => {
                // Handle if expressions - this is tricky in the visitor pattern
                // For now, return a placeholder
                "if expression".to_string()
            }

            Expr::Match(_expr_match) => {
                // Handle match expressions
                "match expression".to_string()
            }

            Expr::Return(expr_return) => {
                if let Some(val) = &expr_return.expr {
                    let val_str = self.expr_to_string(val);
                    format!("return {}", val_str)
                } else {
                    "return".to_string()
                }
            }

            Expr::Break(_) => "exit loop".to_string(),

            Expr::Continue(_) => "continue loop".to_string(),

            Expr::Reference(expr_ref) => {
                let inner = self.expr_to_string(&expr_ref.expr);
                if expr_ref.mutability.is_some() {
                    format!("mutable reference to {}", inner)
                } else {
                    format!("reference to {}", inner)
                }
            }

            Expr::Tuple(expr_tuple) => {
                let elems: Vec<String> = expr_tuple
                    .elems
                    .iter()
                    .map(|e| self.expr_to_string(e))
                    .collect();
                format!("tuple of {}", elems.join(" and "))
            }

            Expr::Array(expr_array) => {
                let elems: Vec<String> = expr_array
                    .elems
                    .iter()
                    .map(|e| self.expr_to_string(e))
                    .collect();
                format!("array of {}", elems.join(" and "))
            }

            Expr::Block(_expr_block) => "block expression".to_string(),

            Expr::Assign(expr_assign) => {
                let left = self.expr_to_string(&expr_assign.left);
                let right = self.expr_to_string(&expr_assign.right);
                format!("set {} equal to {}", left, right)
            }

            Expr::Paren(expr_paren) => self.expr_to_string(&expr_paren.expr),

            Expr::Try(expr_try) => {
                let inner = self.expr_to_string(&expr_try.expr);
                format!("{} unwrap or return error", inner)
            }

            Expr::Closure(expr_closure) => {
                // Extract closure parameters
                let params: Vec<String> = expr_closure
                    .inputs
                    .iter()
                    .map(|pat| match pat {
                        Pat::Ident(pat_ident) => {
                            let name = pat_ident.ident.to_string();
                            if pat_ident.mutability.is_some() {
                                format!("mutable {}", sanitize_identifier(&name))
                            } else {
                                sanitize_identifier(&name)
                            }
                        }
                        Pat::Type(pat_type) => {
                            // Handle typed parameter: |x: i32|
                            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                                let name = pat_ident.ident.to_string();
                                let ty = map_type_to_iron(&pat_type.ty);
                                format!("{} of {}", sanitize_identifier(&name), ty)
                            } else {
                                "param".to_string()
                            }
                        }
                        _ => "param".to_string(),
                    })
                    .collect();

                // Check for move keyword
                let move_prefix = if expr_closure.movability.is_some() {
                    "move "
                } else {
                    ""
                };

                // Handle closure body
                let body_str = match &*expr_closure.body {
                    Expr::Block(block) => {
                        // Multi-statement closure body
                        let stmts: Vec<String> = block
                            .block
                            .stmts
                            .iter()
                            .map(|stmt| self.stmt_to_string(stmt))
                            .collect();
                        stmts.join(" ")
                    }
                    expr => {
                        // Single expression closure body
                        self.expr_to_string(expr)
                    }
                };

                if params.is_empty() {
                    format!("{}closure with body {}", move_prefix, body_str)
                } else {
                    format!(
                        "{}closure with parameters {} and body {}",
                        move_prefix,
                        params.join(" and "),
                        body_str
                    )
                }
            }

            Expr::Index(expr_index) => {
                let base = self.expr_to_string(&expr_index.expr);
                let idx = self.expr_to_string(&expr_index.index);
                format!("index {} at {}", base, idx)
            }

            Expr::Struct(expr_struct) => {
                let name = expr_struct
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let ident_name = sanitize_identifier(&name);

                if expr_struct.fields.is_empty() {
                    format!("create {}", ident_name)
                } else {
                    let fields: Vec<String> = expr_struct
                        .fields
                        .iter()
                        .map(|field| {
                            let field_name = match &field.member {
                                syn::Member::Named(ident) => ident.to_string(),
                                syn::Member::Unnamed(_) => "field".to_string(),
                            };
                            let value = self.expr_to_string(&field.expr);
                            format!("{} of {}", sanitize_identifier(&field_name), value)
                        })
                        .collect();
                    format!("create {} with {}", ident_name, fields.join(" and "))
                }
            }

            Expr::Range(expr_range) => {
                let start = expr_range
                    .start
                    .as_ref()
                    .map(|e| self.expr_to_string(e))
                    .unwrap_or_else(|| "start".to_string());
                let end = expr_range
                    .end
                    .as_ref()
                    .map(|e| self.expr_to_string(e))
                    .unwrap_or_else(|| "end".to_string());

                // Use syn::RangeLimits properly
                if matches!(expr_range.limits, syn::RangeLimits::HalfOpen(_)) {
                    format!("range from {} to {}", start, end)
                } else {
                    format!("inclusive range from {} to {}", start, end)
                }
            }

            Expr::Macro(expr_macro) => {
                // Extract macro name
                let name = expr_macro
                    .mac
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Extract macro arguments as raw tokens
                let args = expr_macro.mac.tokens.to_string();

                // Check delimiter type (brackets [] vs parentheses ())
                let uses_brackets =
                    matches!(expr_macro.mac.delimiter, syn::MacroDelimiter::Bracket(_));
                let bracket_suffix = if uses_brackets { " bracket" } else { "" };

                if args.is_empty() {
                    format!("macro {}{}", sanitize_identifier(&name), bracket_suffix)
                } else {
                    format!(
                        "macro {} with {}{}",
                        sanitize_identifier(&name),
                        args, // Don't sanitize macro args, preserve exact syntax
                        bracket_suffix
                    )
                }
            }

            _ => {
                format!("unsupported expression: {:?}", expr)
            }
        }
    }

    /// Convert a statement to string representation for closure bodies
    fn stmt_to_string(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    match &local.pat {
                        Pat::Ident(pat_ident) => {
                            let name = pat_ident.ident.to_string();
                            let value = self.expr_to_string(&init.expr);
                            if pat_ident.mutability.is_some() {
                                format!(
                                    "define mutable {} as {}",
                                    sanitize_identifier(&name),
                                    value
                                )
                            } else {
                                format!("define {} as {}", sanitize_identifier(&name), value)
                            }
                        }
                        _ => "statement".to_string(),
                    }
                } else {
                    "statement".to_string()
                }
            }
            Stmt::Expr(expr, _) => self.expr_to_string(expr),
            _ => "statement".to_string(),
        }
    }

    /// Emit a for loop
    fn emit_for_loop(&mut self, for_loop: &syn::ExprForLoop) {
        // Get the pattern (loop variable)
        let var_name = match &*for_loop.pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            _ => "item".to_string(),
        };

        // Get the iterator expression
        let iterator = self.expr_to_string(&for_loop.expr);

        // Emit the for header
        self.emitter.write_for_header(&var_name, &iterator);

        // Emit the body
        self.emitter.begin_block();
        for stmt in &for_loop.body.stmts {
            self.visit_stmt(stmt);
        }
        self.emitter.end_for();
    }

    /// Emit a while loop
    fn emit_while_loop(&mut self, while_loop: &syn::ExprWhile) {
        // Get the condition
        let condition = self.expr_to_string(&while_loop.cond);

        // Emit the while header
        self.emitter.write_while_header(&condition);

        // Emit the body
        self.emitter.begin_block();
        for stmt in &while_loop.body.stmts {
            self.visit_stmt(stmt);
        }
        self.emitter.end_while();
    }

    /// Emit an if statement
    fn emit_if_statement(&mut self, if_expr: &syn::ExprIf) {
        // Get the condition
        let condition = self.expr_to_string(&if_expr.cond);

        // Emit the if header
        self.emitter.write_if_header(&condition);

        // Emit the then block
        self.emitter.begin_block();
        for stmt in &if_expr.then_branch.stmts {
            self.visit_stmt(stmt);
        }
        self.emitter.end_if();

        // Handle else branch if present
        if let Some((_, else_branch)) = &if_expr.else_branch {
            self.emitter.write_else();
            self.emitter.begin_block();
            // The else branch can be another if or a block
            match &**else_branch {
                Expr::If(nested_if) => {
                    self.emit_if_statement(nested_if);
                }
                Expr::Block(block) => {
                    for stmt in &block.block.stmts {
                        self.visit_stmt(stmt);
                    }
                    self.emitter.end_if();
                }
                _ => {
                    let else_str = self.expr_to_string(else_branch);
                    if !else_str.is_empty() {
                        self.emitter.write_line(&else_str);
                    }
                    self.emitter.end_if();
                }
            }
        }
    }
}

impl Default for IronParser {
    fn default() -> Self {
        Self::new()
    }
}
