//! Iron Abstract Syntax Tree definitions

#[derive(Debug, Clone)]
pub enum IronType {
    Named(String),
    Reference(Box<IronType>),
    MutableReference(Box<IronType>),
    RawPointer(Box<IronType>),
    MutableRawPointer(Box<IronType>),
    Optional(Box<IronType>),
    Result(Box<IronType>, Box<IronType>),
    List(Box<IronType>),
    BoxType(Box<IronType>),
    Tuple(Vec<IronType>),
    Array(Box<IronType>),
    Slice(Box<IronType>),
    Function(Vec<IronType>, Box<IronType>), // params, return
    Generic(String, Vec<IronBound>),
}

#[derive(Debug, Clone)]
pub struct IronBound {
    pub trait_name: String,
}

#[derive(Debug, Clone)]
pub struct IronParam {
    pub name: String,
    pub ty: IronType,
}

#[derive(Debug, Clone)]
pub struct IronField {
    pub name: String,
    pub ty: IronType,
}

#[derive(Debug, Clone)]
pub struct IronVariant {
    pub name: String,
    pub data: Option<IronVariantData>,
}

#[derive(Debug, Clone)]
pub enum IronVariantData {
    Type(IronType),
    Fields(Vec<IronField>),
}

#[derive(Debug, Clone)]
pub struct IronGeneric {
    pub name: String,
    pub bounds: Vec<IronBound>,
}

#[derive(Debug, Clone)]
pub enum IronExpr {
    Identifier(String),
    String(String),
    Integer(String),
    Float(String),
    Boolean(bool),
    Binary {
        left: Box<IronExpr>,
        op: IronBinaryOp,
        right: Box<IronExpr>,
    },
    Unary {
        op: IronUnaryOp,
        expr: Box<IronExpr>,
    },
    Call {
        func: Box<IronExpr>,
        args: Vec<IronExpr>,
    },
    MethodCall {
        receiver: Box<IronExpr>,
        method: String,
        args: Vec<IronExpr>,
    },
    AssociatedFunctionCall {
        type_name: String,
        function: String,
        args: Vec<IronExpr>,
    },
    Macro {
        name: String,
        args: String,
        bracket: bool, // true for [], false for () or {}
    },
    FieldAccess {
        base: Box<IronExpr>,
        field: String,
    },
    Try {
        expr: Box<IronExpr>,
    },
    Some(Box<IronExpr>),
    None,
    Ok(Box<IronExpr>),
    Err(Box<IronExpr>),
    Tuple(Vec<IronExpr>),
    Array(Vec<IronExpr>),
    Struct {
        name: String,
        fields: Vec<(IronField, IronExpr)>,
    },
    Index {
        base: Box<IronExpr>,
        index: Box<IronExpr>,
    },
    Range {
        start: Option<Box<IronExpr>>,
        end: Option<Box<IronExpr>>,
        inclusive: bool,
    },
    Closure {
        params: Vec<IronParam>,
        body: Vec<IronStmt>,
    },
}

#[derive(Debug, Clone)]
pub enum IronBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone)]
pub enum IronUnaryOp {
    Not,
    Neg,
    Deref,
}

#[derive(Debug, Clone)]
pub enum IronStmt {
    Let {
        name: String,
        mutable: bool,
        value: IronExpr,
    },
    Assign {
        target: IronExpr,
        value: IronExpr,
    },
    Expr(IronExpr),
    Return(Option<IronExpr>),
    Break,
    Continue,
    If {
        condition: IronExpr,
        then_block: Vec<IronStmt>,
        else_block: Option<Vec<IronStmt>>,
    },
    While {
        condition: IronExpr,
        body: Vec<IronStmt>,
    },
    For {
        var: String,
        iterator: IronExpr,
        body: Vec<IronStmt>,
    },
    Match {
        expr: IronExpr,
        arms: Vec<(IronPattern, IronExpr)>,
    },
}

#[derive(Debug, Clone)]
pub enum IronPattern {
    Identifier(String),
    Wildcard,
    Literal(IronExpr),
    Tuple(Vec<IronPattern>),
    Struct {
        name: String,
        fields: Vec<(IronField, IronPattern)>,
    },
    Variant {
        enum_name: String,
        variant_name: String,
        data: Option<Box<IronPattern>>,
    },
}

#[derive(Debug, Clone)]
pub struct IronFunction {
    pub name: String,
    pub generics: Vec<IronGeneric>,
    pub params: Vec<IronParam>,
    pub return_type: Option<IronType>,
    pub body: Vec<IronStmt>,
}

#[derive(Debug, Clone)]
pub struct IronStruct {
    pub name: String,
    pub generics: Vec<IronGeneric>,
    pub fields: Vec<IronField>,
}

#[derive(Debug, Clone)]
pub struct IronEnum {
    pub name: String,
    pub generics: Vec<IronGeneric>,
    pub variants: Vec<IronVariant>,
}

#[derive(Debug, Clone)]
pub struct IronStatic {
    pub name: String,
    pub mutable: bool,
    pub ty: IronType,
    pub value: IronExpr,
}

#[derive(Debug, Clone)]
pub struct IronConst {
    pub name: String,
    pub ty: IronType,
    pub value: IronExpr,
}

#[derive(Debug, Clone)]
pub struct IronTypeAlias {
    pub name: String,
    pub generics: Vec<IronGeneric>,
    pub ty: IronType,
}

#[derive(Debug, Clone)]
pub enum IronItem {
    Function(IronFunction),
    Struct(IronStruct),
    Enum(IronEnum),
    Static(IronStatic),
    Const(IronConst),
    TypeAlias(IronTypeAlias),
    Verbatim(String),
}

#[derive(Debug, Clone)]
pub struct IronFile {
    pub items: Vec<IronItem>,
}
