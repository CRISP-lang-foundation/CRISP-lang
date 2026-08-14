// Copyright 2026 Peter Leukanič
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! AST - Abstract Syntax Tree

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        sigil: VarType,
        expr: Option<Box<Expr>>,
        mutable: bool,
    },
    Assign {
        name: String,
        expr: Box<Expr>,
    },
    Const {
        name: String,
        sigil: VarType,
        expr: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Vec<Stmt>,
    },
    For {
        variable: String,
        iterable: Box<Expr>,
        body: Vec<Stmt>,
    },
    Return(Option<Box<Expr>>),
    Break,
    Continue,
    Expr(Box<Expr>),
    Say(Vec<Expr>),
    Print(Vec<Expr>),
    Warn(Vec<Expr>),
    Die(Box<Expr>),
    Throw(Box<Expr>),
    Use(String),
    Block(Vec<Stmt>),
    Fn {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    TryCatch {
        try_block: Vec<Stmt>,
        catch_var: Option<String>,
        catch_block: Option<Vec<Stmt>>,
        finally_block: Option<Vec<Stmt>>,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    // --- OOP ---
    Class {
        name: String,
        parent: Option<String>,
        methods: Vec<Method>,
        static_methods: Vec<Method>,
    },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<Box<Expr>>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum MatchPattern {
    Var(String),
    Int(i64),
    Str(String),
    Bool(bool),
    Wildcard,
}

#[derive(Debug, Clone)]
pub enum VarType {
    Scalar,
    Array,
    Hash,   // Added hash type
    Ref,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub sigil: VarType,
    pub type_hint: Option<String>,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Regex(String),
    Bool(bool),
    Null,
    Var {
        name: String,
        sigil: Option<VarType>,
    },
    // --- ASSIGNMENT ---
    Assign {
        name: String,
        sigil: Option<VarType>,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Match {
        left: Box<Expr>,
        right: Box<Expr>,
        global: bool,
    },
    Substitute {
        text: Box<Expr>,
        pattern: Box<Expr>,
        replacement: Box<Expr>,
        global: bool,
    },
    Array(Vec<Expr>),
    Hash(Vec<(Expr, Expr)>),
    Index {
        collection: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        collection: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    HashAccess {
        hash: Box<Expr>,
        key: Box<Expr>,
    },
    Ref(Box<Expr>),
    Deref(Box<Expr>),
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        named_args: Vec<(String, Expr)>,
    },
    Method {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    Concat {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Repeat {
        expr: Box<Expr>,
        count: Box<Expr>,
    },
    Spaceship {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FatComma {
        key: Box<Expr>,
        value: Box<Expr>,
    },
    Where {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    
    Object {
        class: String,
        args: Vec<Expr>,
        named_args: Vec<(String, Expr)>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FieldAssign {
        object: Box<Expr>,
        field: String,
        value: Box<Expr>,
    },
    SelfRef,
    SuperRef,
}

impl Expr {
    pub fn as_str(&self) -> String {
        match self {
            Expr::Str(s) => s.clone(),
            Expr::Var { name, .. } => name.clone(),
            Expr::Int(i) => i.to_string(),
            Expr::Float(f) => f.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Null => "null".to_string(),
            _ => format!("{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Cmp,
    And,
    Or,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    Concat,
    Repeat,
    Match,      // Added =~ operator
    NotMatch,   // Added !~ operator
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,
    Deref,
    BitNot,
}
