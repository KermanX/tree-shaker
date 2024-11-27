use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  dep::{DepId, ReferredDeps},
  scope::conditional::ConditionalDataMap,
  utils::{DataPlaceholder, ExtraData, StatementVecData},
  TreeShakeConfig,
};
use oxc::{
  allocator::{Allocator, CloneIn},
  ast::{
    ast::{
      ArrayExpressionElement, AssignmentTarget, BinaryOperator, BindingIdentifier, BindingPattern,
      BindingPatternKind, Expression, ForStatementLeft, FormalParameterKind, IdentifierReference,
      LogicalOperator, NumberBase, Program, SimpleAssignmentTarget, Statement, UnaryOperator,
      VariableDeclarationKind,
    },
    AstBuilder, NONE,
  },
  semantic::{ScopeId, Semantic, SymbolId},
  span::{GetSpan, Span, SPAN},
};
use rustc_hash::FxHashMap;
use std::{
  cell::{Cell, RefCell},
  hash::{DefaultHasher, Hasher},
  mem,
};

pub struct Transformer<'a> {
  pub config: &'a TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub semantic: Semantic<'a>,
  pub ast_builder: AstBuilder<'a>,
  pub data: ExtraData<'a>,
  pub referred_deps: ReferredDeps,
  pub conditional_data: ConditionalDataMap<'a>,
  pub var_decls: RefCell<FxHashMap<SymbolId, bool>>,

  /// The block statement has already exited, so we can and only can transform declarations themselves
  pub declaration_only: Cell<bool>,
  pub need_unused_assignment_target: Cell<bool>,
  pub need_non_nullish_helper: Cell<bool>,
  pub unused_identifier_names: RefCell<FxHashMap<u64, usize>>,
}

impl<'a> Transformer<'a> {
  pub fn new(analyzer: Analyzer<'a>) -> Self {
    let Analyzer {
      config,
      allocator,
      semantic,
      data,
      referred_deps: referred_nodes,
      conditional_data,
      ..
    } = analyzer;

    // let mut counts: Vec<_> = referred_nodes.clone().into_iter().collect();
    // counts.sort_by(|a, b| b.1.cmp(&a.1));
    // for (key, v) in counts {
    //   if v > 10 {
    //     println!("{key:?}: {v}");
    //   }
    // }
    // println!("---");

    Transformer {
      config,
      allocator,
      semantic,
      ast_builder: AstBuilder::new(allocator),
      data,
      referred_deps: referred_nodes,
      conditional_data,
      var_decls: Default::default(),

      declaration_only: Cell::new(false),
      need_unused_assignment_target: Cell::new(false),
      need_non_nullish_helper: Cell::new(false),
      unused_identifier_names: Default::default(),
    }
  }

  pub fn transform_program(&self, node: &'a Program<'a>) -> Program<'a> {
    let Program { span, source_type, source_text, comments, hashbang, directives, body, .. } = node;

    let data = self.get_data::<StatementVecData>(AstKind2::Program(node));
    let mut body = self.transform_statement_vec(data, body);

    self.patch_var_declarations(node.scope_id.get().unwrap(), &mut body);

    if self.need_unused_assignment_target.get() {
      body.push(self.build_unused_assignment_target_definition());
    }
    if self.need_non_nullish_helper.get() {
      body.push(self.build_non_nullish_helper_definition());
    }

    self.ast_builder.program(
      *span,
      *source_type,
      *source_text,
      self.clone_node(comments),
      self.clone_node(hashbang),
      self.clone_node(directives),
      body,
    )
  }

  pub fn update_var_decl_state(&self, symbol: SymbolId, is_declaration: bool) {
    if !self.semantic.symbols().get_flags(symbol).is_function_scoped_declaration() {
      return;
    }
    let mut var_decls = self.var_decls.borrow_mut();
    if is_declaration {
      var_decls.insert(symbol, false);
    } else {
      var_decls.entry(symbol).or_insert(true);
    }
  }

  /// Append missing var declarations at the end of the function body or program
  pub fn patch_var_declarations(
    &self,
    scope_id: ScopeId,
    statements: &mut oxc::allocator::Vec<'a, Statement<'a>>,
  ) {
    let bindings = self.semantic.scopes().get_bindings(scope_id);
    if bindings.is_empty() {
      return;
    }

    let var_decls = self.var_decls.borrow();
    let mut declarations = self.ast_builder.vec();
    for symbol_id in bindings.values() {
      if var_decls.get(symbol_id) == Some(&true) {
        let name = self.semantic.symbols().get_name(*symbol_id);
        let span = self.semantic.symbols().get_span(*symbol_id);
        declarations.push(self.ast_builder.variable_declarator(
          span,
          VariableDeclarationKind::Var,
          self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_binding_identifier(span, name),
            NONE,
            false,
          ),
          None,
          false,
        ));
      }
    }

    if !declarations.is_empty() {
      statements.push(Statement::from(self.ast_builder.declaration_variable(
        SPAN,
        VariableDeclarationKind::Var,
        declarations,
        false,
      )));
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn clone_node<T: CloneIn<'a>>(&self, node: &T) -> T::Cloned {
    node.clone_in(self.allocator)
  }

  pub fn build_unused_binding_identifier(&self, span: Span) -> BindingIdentifier<'a> {
    let text = self.semantic.source_text().as_bytes();
    let start = 5.max(span.start as usize) - 5;
    let end = text.len().min(span.end as usize + 5);

    let mut hasher = DefaultHasher::new();
    hasher.write(&text[start..end]);
    let hash = hasher.finish() % 0xFFFF;
    let index =
      *self.unused_identifier_names.borrow_mut().entry(hash).and_modify(|e| *e += 1).or_insert(0);
    let name = if index == 0 {
      format!("__unused_{:04X}", hash)
    } else {
      format!("__unused_{:04X}_{}", hash, index - 1)
    };
    self.ast_builder.binding_identifier(span, name)
  }

  pub fn build_unused_binding_pattern(&self, span: Span) -> BindingPattern<'a> {
    self.ast_builder.binding_pattern(
      BindingPatternKind::BindingIdentifier(
        self.ast_builder.alloc(self.build_unused_binding_identifier(span)),
      ),
      NONE,
      false,
    )
  }

  pub fn build_unused_assignment_binding_pattern(&self, span: Span) -> BindingPattern<'a> {
    self.ast_builder.binding_pattern(
      self.ast_builder.binding_pattern_kind_assignment_pattern(
        span,
        self.build_unused_binding_pattern(SPAN),
        self.build_unused_expression(SPAN),
      ),
      NONE,
      false,
    )
  }

  pub fn build_unused_identifier_reference_write(&self, span: Span) -> IdentifierReference<'a> {
    self.need_unused_assignment_target.set(true);
    self.ast_builder.identifier_reference(span, "__unused__")
  }

  pub fn build_unused_simple_assignment_target(&self, span: Span) -> SimpleAssignmentTarget<'a> {
    SimpleAssignmentTarget::AssignmentTargetIdentifier(
      self.ast_builder.alloc(self.build_unused_identifier_reference_write(span)),
    )
  }

  pub fn build_unused_assignment_target(&self, span: Span) -> AssignmentTarget<'a> {
    // The commented doesn't work because nullish value can't be destructured
    // self.ast_builder.assignment_target_assignment_target_pattern(
    //   self.ast_builder.assignment_target_pattern_object_assignment_target(
    //     span,
    //     self.ast_builder.vec(),
    //     None,
    //   ),
    // )
    AssignmentTarget::from(self.build_unused_simple_assignment_target(span))
  }

  pub fn build_unused_assignment_target_in_rest(&self, span: Span) -> AssignmentTarget<'a> {
    AssignmentTarget::from(self.build_unused_simple_assignment_target(span))
  }

  pub fn build_unused_for_statement_left(&self, span: Span) -> ForStatementLeft<'a> {
    ForStatementLeft::from(self.build_unused_assignment_target(span))
  }

  pub fn build_unused_expression(&self, span: Span) -> Expression<'a> {
    self.ast_builder.expression_numeric_literal(span, 0.0f64, "0", NumberBase::Decimal)
  }

  pub fn build_unused_iterable(&self, span: Span, length: usize) -> Expression<'a> {
    let mut elements = self.ast_builder.vec();
    for _ in 0..length {
      elements.push(ArrayExpressionElement::from(self.build_unused_expression(SPAN)));
    }
    self.ast_builder.expression_array(span, elements, None)
  }

  pub fn build_undefined(&self, span: Span) -> Expression<'a> {
    self.ast_builder.expression_identifier_reference(span, "undefined")
  }

  pub fn build_negate_expression(&self, expression: Expression<'a>) -> Expression<'a> {
    self.ast_builder.expression_unary(expression.span(), UnaryOperator::LogicalNot, expression)
  }

  pub fn build_object_spread_effect(&self, span: Span, argument: Expression<'a>) -> Expression<'a> {
    self.ast_builder.expression_object(
      span,
      self.ast_builder.vec1(self.ast_builder.object_property_kind_spread_element(span, argument)),
      None,
    )
  }

  pub fn build_unused_assignment_target_definition(&self) -> Statement<'a> {
    Statement::from(self.ast_builder.declaration_variable(
      SPAN,
      VariableDeclarationKind::Var,
      self.ast_builder.vec1(self.ast_builder.variable_declarator(
        SPAN,
        VariableDeclarationKind::Var,
        self.ast_builder.binding_pattern(
          self.ast_builder.binding_pattern_kind_binding_identifier(SPAN, "__unused__"),
          NONE,
          false,
        ),
        None,
        false,
      )),
      false,
    ))
  }

  pub fn build_non_nullish_helper_definition(&self) -> Statement<'a> {
    Statement::from(self.ast_builder.declaration_variable(
      SPAN,
      VariableDeclarationKind::Var,
      self.ast_builder.vec1(self.ast_builder.variable_declarator(
        SPAN,
        VariableDeclarationKind::Var,
        self.ast_builder.binding_pattern(
          self.ast_builder.binding_pattern_kind_binding_identifier(SPAN, "__non_nullish__"),
          NONE,
          false,
        ),
        Some(self.ast_builder.expression_arrow_function(
          SPAN,
          true,
          false,
          NONE,
          self.ast_builder.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            self.ast_builder.vec1(self.ast_builder.formal_parameter(
              SPAN,
              self.ast_builder.vec(),
              self.ast_builder.binding_pattern(
                self.ast_builder.binding_pattern_kind_binding_identifier(SPAN, "v"),
                NONE,
                false,
              ),
              None,
              false,
              false,
            )),
            NONE,
          ),
          NONE,
          self.ast_builder.function_body(
            SPAN,
            self.ast_builder.vec(),
            self.ast_builder.vec1(self.ast_builder.statement_expression(
              SPAN,
              self.ast_builder.expression_logical(
                SPAN,
                self.ast_builder.expression_binary(
                  SPAN,
                  self.ast_builder.expression_identifier_reference(SPAN, "v"),
                  BinaryOperator::StrictInequality,
                  self.ast_builder.expression_null_literal(SPAN),
                ),
                LogicalOperator::And,
                self.ast_builder.expression_binary(
                  SPAN,
                  self.ast_builder.expression_identifier_reference(SPAN, "v"),
                  BinaryOperator::StrictInequality,
                  self.ast_builder.expression_identifier_reference(SPAN, "undefined"),
                ),
              ),
            )),
          ),
        )),
        false,
      )),
      false,
    ))
  }

  pub fn build_chain_expression_mock(
    &self,
    span: Span,
    left: Expression<'a>,
    right: Expression<'a>,
  ) -> Expression<'a> {
    self.need_non_nullish_helper.set(true);
    self.ast_builder.expression_logical(
      span,
      self.ast_builder.expression_call(
        left.span(),
        self.ast_builder.expression_identifier_reference(span, "__non_nullish__"),
        NONE,
        self.ast_builder.vec1(left.into()),
        false,
      ),
      LogicalOperator::And,
      right,
    )
  }
}

impl<'a> Transformer<'a> {
  pub fn get_data<D: Default + 'a>(&self, key: impl Into<DepId>) -> &'a D {
    let existing = self.data.get(&key.into());
    match existing {
      Some(boxed) => unsafe { mem::transmute::<&DataPlaceholder<'_>, &D>(boxed.as_ref()) },
      None => self.allocator.alloc(D::default()),
    }
  }
}
