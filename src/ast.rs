use oxc::{allocator::Vec, ast::ast::Argument};

pub type Arguments<'a> = Vec<'a, Argument<'a>>;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AstType2 {
  BooleanLiteral,
  NullLiteral,
  NumericLiteral,
  BigIntLiteral,
  RegExpLiteral,
  StringLiteral,
  Program,
  IdentifierName,
  IdentifierReference,
  BindingIdentifier,
  LabelIdentifier,
  ThisExpression,
  ArrayExpression,
  ArrayExpressionElement,
  Elision,
  ObjectExpression,
  ObjectProperty,
  PropertyKey,
  TemplateLiteral,
  TaggedTemplateExpression,
  MemberExpression,
  CallExpression,
  NewExpression,
  MetaProperty,
  SpreadElement,
  Argument,
  UpdateExpression,
  UnaryExpression,
  BinaryExpression,
  PrivateInExpression,
  LogicalExpression,
  ConditionalExpression,
  AssignmentExpression,
  AssignmentTarget,
  SimpleAssignmentTarget,
  AssignmentTargetPattern,
  ArrayAssignmentTarget,
  ObjectAssignmentTarget,
  AssignmentTargetWithDefault,
  SequenceExpression,
  Super,
  AwaitExpression,
  ChainExpression,
  ParenthesizedExpression,
  Directive,
  Hashbang,
  BlockStatement,
  VariableDeclaration,
  VariableDeclarator,
  UsingDeclaration,
  EmptyStatement,
  ExpressionStatement,
  IfStatement,
  DoWhileStatement,
  WhileStatement,
  ForStatement,
  ForStatementInit,
  ForInStatement,
  ForOfStatement,
  ContinueStatement,
  BreakStatement,
  ReturnStatement,
  WithStatement,
  SwitchStatement,
  SwitchCase,
  LabeledStatement,
  ThrowStatement,
  TryStatement,
  FinallyClause,
  CatchClause,
  CatchParameter,
  DebuggerStatement,
  AssignmentPattern,
  ObjectPattern,
  ArrayPattern,
  BindingRestElement,
  Function,
  FormalParameters,
  FormalParameter,
  FunctionBody,
  ArrowFunctionExpression,
  YieldExpression,
  Class,
  ClassHeritage,
  ClassBody,
  MethodDefinition,
  PropertyDefinition,
  PrivateIdentifier,
  StaticBlock,
  ModuleDeclaration,
  ImportExpression,
  ImportDeclaration,
  ImportSpecifier,
  ImportDefaultSpecifier,
  ImportNamespaceSpecifier,
  ExportNamedDeclaration,
  ExportDefaultDeclaration,
  ExportAllDeclaration,
  ExportSpecifier,
  TSThisParameter,
  TSEnumDeclaration,
  TSEnumMember,
  TSTypeAnnotation,
  TSLiteralType,
  TSConditionalType,
  TSUnionType,
  TSIntersectionType,
  TSParenthesizedType,
  TSIndexedAccessType,
  TSNamedTupleMember,
  TSAnyKeyword,
  TSStringKeyword,
  TSBooleanKeyword,
  TSNumberKeyword,
  TSNeverKeyword,
  TSIntrinsicKeyword,
  TSUnknownKeyword,
  TSNullKeyword,
  TSUndefinedKeyword,
  TSVoidKeyword,
  TSSymbolKeyword,
  TSThisType,
  TSObjectKeyword,
  TSBigIntKeyword,
  TSTypeReference,
  TSTypeName,
  TSQualifiedName,
  TSTypeParameterInstantiation,
  TSTypeParameter,
  TSTypeParameterDeclaration,
  TSTypeAliasDeclaration,
  TSClassImplements,
  TSInterfaceDeclaration,
  TSPropertySignature,
  TSMethodSignature,
  TSConstructSignatureDeclaration,
  TSInterfaceHeritage,
  TSModuleDeclaration,
  TSModuleBlock,
  TSTypeLiteral,
  TSInferType,
  TSTypeQuery,
  TSImportType,
  TSMappedType,
  TSTemplateLiteralType,
  TSAsExpression,
  TSSatisfiesExpression,
  TSTypeAssertion,
  TSImportEqualsDeclaration,
  TSModuleReference,
  TSExternalModuleReference,
  TSNonNullExpression,
  Decorator,
  TSExportAssignment,
  TSInstantiationExpression,
  JSXElement,
  JSXOpeningElement,
  JSXClosingElement,
  JSXFragment,
  JSXElementName,
  JSXNamespacedName,
  JSXMemberExpression,
  JSXMemberExpressionObject,
  JSXExpressionContainer,
  JSXAttributeItem,
  JSXSpreadAttribute,
  JSXIdentifier,
  JSXText,
  ExpressionArrayElement,

  // extras
  BindingPattern,
  Declaration,
  Expression,
  Statement,
  Arguments,
  CaseConsequent,
  MemberExpressionRead,
  MemberExpressionWrite,
  SimpleAssignmentTargetRead,
  SimpleAssignmentTargetWrite,
  AssignmentTargetProperty,
  AssignmentTargetPropertyIdentifier,
  AssignmentTargetRest,
  BindingProperty,
  Callee,
  ExpressionInTaggedTemplate,
  LogicalExpressionLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeclarationKind {
  Var,
  UntrackedVar,
  Let,
  Const,
  Function,
  NamedFunctionInBody,
  Class,
  Import,
  Caught,
  FunctionParameter,
  ArrowFunctionParameter,
}

impl DeclarationKind {
  pub fn is_var(self) -> bool {
    matches!(self, DeclarationKind::Var | DeclarationKind::UntrackedVar)
  }

  pub fn is_untracked(self) -> bool {
    matches!(self, DeclarationKind::UntrackedVar)
  }

  pub fn is_const(self) -> bool {
    matches!(self, DeclarationKind::Const | DeclarationKind::NamedFunctionInBody)
  }

  pub fn is_redeclarable(self) -> bool {
    matches!(
      self,
      DeclarationKind::Var
        | DeclarationKind::UntrackedVar
        | DeclarationKind::Function
        | DeclarationKind::Class
    )
  }

  pub fn is_shadowable(self) -> bool {
    self.is_redeclarable()
      || matches!(
        self,
        DeclarationKind::FunctionParameter
          | DeclarationKind::ArrowFunctionParameter
          | DeclarationKind::Caught
      )
  }
}
