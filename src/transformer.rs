use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  data::{
    get_node_ptr, DataPlaceholder, ExtraData, ReferredNodes, StatementVecData, VarDeclarations,
  },
  entity::EntityDepNode,
  logger::Logger,
  TreeShakeConfig,
};
use oxc::{
  allocator::{Allocator, CloneIn},
  ast::{
    ast::{
      AssignmentTarget, BindingPattern, Expression, ForStatementLeft, IdentifierReference,
      NumberBase, Program, SimpleAssignmentTarget, Statement, UnaryOperator,
      VariableDeclarationKind,
    },
    AstBuilder, NONE,
  },
  semantic::Semantic,
  span::{GetSpan, Span, SPAN},
};
use std::{
  cell::{Cell, RefCell},
  hash::{DefaultHasher, Hasher},
  mem,
};

pub struct Transformer<'a> {
  pub config: TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub semantic: Semantic<'a>,
  pub ast_builder: AstBuilder<'a>,
  pub data: ExtraData<'a>,
  pub referred_nodes: RefCell<ReferredNodes<'a>>,
  pub var_decls: RefCell<VarDeclarations<'a>>,
  pub logger: Option<&'a Logger>,

  pub call_stack: RefCell<Vec<EntityDepNode>>,
  pub need_unused_assignment_target: Cell<bool>,
}

impl<'a> Transformer<'a> {
  pub fn new(analyzer: Analyzer<'a>) -> Self {
    let Analyzer { config, allocator, semantic, data, referred_nodes, var_decls, logger, .. } =
      analyzer;

    Transformer {
      config,
      allocator,
      semantic,
      ast_builder: AstBuilder::new(allocator),
      data,
      referred_nodes: RefCell::new(referred_nodes),
      var_decls: RefCell::new(var_decls),
      logger,

      call_stack: RefCell::new(vec![EntityDepNode::Environment]),
      need_unused_assignment_target: Cell::new(false),
    }
  }

  pub fn transform_program(&self, node: &'a Program<'a>) -> Program<'a> {
    let Program { span, source_type, hashbang, directives, body, .. } = node;

    let data = self.get_data::<StatementVecData>(AstType2::Program, node);
    let mut body = self.transform_statement_vec(data, body);

    self.patch_var_declarations(&mut body);

    if self.need_unused_assignment_target.get() {
      body.push(self.ast_builder.statement_declaration(self.ast_builder.declaration_variable(
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
      )));
    }

    self.ast_builder.program(
      *span,
      *source_type,
      self.clone_node(hashbang),
      self.clone_node(directives),
      body,
    )
  }

  /// Append missing var declarations at the end of the function body or program
  pub fn patch_var_declarations(&self, statements: &mut oxc::allocator::Vec<'a, Statement<'a>>) {
    let call_stack = self.call_stack.borrow();
    let key = call_stack.last().unwrap();
    if let Some(var_decls) = self.var_decls.borrow().get(key) {
      if !var_decls.is_empty() {
        statements.push(self.ast_builder.statement_declaration(
          self.ast_builder.declaration_variable(
            SPAN,
            VariableDeclarationKind::Var,
            {
              let mut decls = self.ast_builder.vec();
              for symbol_id in var_decls.iter() {
                let name = self.semantic.symbols().get_name(*symbol_id);
                let span = self.semantic.symbols().get_span(*symbol_id);
                decls.push(self.ast_builder.variable_declarator(
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
              decls
            },
            false,
          ),
        ));
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn clone_node<T: CloneIn<'a>>(&self, node: &T) -> T::Cloned {
    node.clone_in(self.allocator)
  }

  pub fn build_unused_binding_identifier(&self, span: Span) -> BindingPattern<'a> {
    let mut hasher = DefaultHasher::new();
    hasher.write_u32(span.start);
    hasher.write_u32(span.end);
    let name = format!("__unused_{:04X}", hasher.finish() % 0xFFFF);
    self.ast_builder.binding_pattern(
      self.ast_builder.binding_pattern_kind_binding_identifier(span, name),
      NONE,
      false,
    )
  }

  pub fn build_unused_binding_pattern(&self, span: Span) -> BindingPattern<'a> {
    self.build_unused_binding_identifier(span)
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
    self.ast_builder.simple_assignment_target_from_identifier_reference(
      self.build_unused_identifier_reference_write(span),
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
    self.ast_builder.assignment_target_simple(self.build_unused_simple_assignment_target(span))
  }

  pub fn build_unused_assignment_target_in_rest(&self, span: Span) -> AssignmentTarget<'a> {
    self.ast_builder.assignment_target_simple(self.build_unused_simple_assignment_target(span))
  }

  pub fn build_unused_for_statement_left(&self, span: Span) -> ForStatementLeft<'a> {
    self.ast_builder.for_statement_left_assignment_target(self.build_unused_assignment_target(span))
  }

  pub fn build_unused_expression(&self, span: Span) -> Expression<'a> {
    self.ast_builder.expression_numeric_literal(span, 0.0f64, "0", NumberBase::Decimal)
  }

  pub fn build_unused_iterable(&self, span: Span, length: usize) -> Expression<'a> {
    let mut elements = self.ast_builder.vec();
    for _ in 0..length {
      elements.push(
        self.ast_builder.array_expression_element_expression(self.build_unused_expression(SPAN)),
      );
    }
    self.ast_builder.expression_array(span, elements, None)
  }

  pub fn build_undefined(&self, span: Span) -> Expression<'a> {
    self.ast_builder.expression_identifier_reference(span, "undefined")
  }

  pub fn build_negate_expression(&self, expression: Expression<'a>) -> Expression<'a> {
    self.ast_builder.expression_unary(expression.span(), UnaryOperator::LogicalNot, expression)
  }
}

impl<'a> Transformer<'a> {
  pub fn get_data<D: Default + 'a>(&self, ast_type: AstType2, node: &impl GetSpan) -> &'a D {
    let key = (ast_type, get_node_ptr(node));
    let existing = self.data.get(&key);
    match existing {
      Some(boxed) => unsafe { mem::transmute::<&DataPlaceholder<'_>, &D>(boxed.as_ref()) },
      None => self.allocator.alloc(D::default()),
    }
  }
}
