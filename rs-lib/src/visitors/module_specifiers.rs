// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use std::collections::HashMap;
use std::path::PathBuf;

use deno_ast::swc::common::BytePos;
use deno_ast::swc::common::Span;
use deno_ast::swc::common::Spanned;
use deno_ast::view::*;
use deno_ast::ModuleSpecifier;

use crate::graph::ModuleGraph;
use crate::mappings::Mappings;
use crate::text_changes::TextChange;
use crate::utils::get_relative_specifier;

pub struct GetModuleSpecifierTextChangesParams<'a> {
  pub specifier: &'a ModuleSpecifier,
  pub module_graph: &'a ModuleGraph,
  pub mappings: &'a Mappings,
  pub program: &'a Program<'a>,
  pub specifier_mappings: &'a HashMap<ModuleSpecifier, String>,
}

struct Context<'a> {
  specifier: &'a ModuleSpecifier,
  module_graph: &'a ModuleGraph,
  mappings: &'a Mappings,
  output_file_path: &'a PathBuf,
  text_changes: Vec<TextChange>,
  specifier_mappings: &'a HashMap<ModuleSpecifier, String>,
}

pub fn get_module_specifier_text_changes(
  params: &GetModuleSpecifierTextChangesParams<'_>,
) -> Vec<TextChange> {
  let mut context = Context {
    specifier: params.specifier,
    module_graph: params.module_graph,
    mappings: params.mappings,
    output_file_path: params.mappings.get_file_path(params.specifier),
    text_changes: Vec::new(),
    specifier_mappings: params.specifier_mappings,
  };

  // todo: look at imports in ts namespaces? I forget if they support importing from another module and if that works in Deno
  for child in params.program.children() {
    match child {
      Node::ImportDecl(import_decl) => {
        visit_module_specifier(import_decl.src, &mut context);
      }
      Node::ExportAll(export_all) => {
        visit_module_specifier(export_all.src, &mut context);
      }
      Node::NamedExport(named_export) => {
        if let Some(src) = named_export.src.as_ref() {
          visit_module_specifier(src, &mut context);
        }
      }
      _ => {}
    }
  }

  context.text_changes
}

fn visit_module_specifier(str: &Str, context: &mut Context) {
  let value = str.value().to_string();
  let specifier = context
    .module_graph
    .resolve_dependency(&value, context.specifier);
  let specifier = match specifier {
    Some(s) => s,
    None => return,
  };

  let new_text =
    if let Some(bare_specifier) = context.specifier_mappings.get(&specifier) {
      bare_specifier.to_string()
    } else {
      let specifier_file_path = context.mappings.get_file_path(&specifier);
      get_relative_specifier(context.output_file_path, specifier_file_path)
    };

  context.text_changes.push(TextChange {
    span: Span::new(
      str.span().lo + BytePos(1),
      str.span().hi - BytePos(1),
      Default::default(),
    ),
    new_text,
  });
}
