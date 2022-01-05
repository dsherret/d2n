// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

use std::collections::HashMap;

use anyhow::Result;
use deno_node_transform::transform;
use deno_node_transform::GlobalName;
use deno_node_transform::MappedSpecifier;
use deno_node_transform::ModuleSpecifier;
use deno_node_transform::Shim;
use deno_node_transform::TransformOptions;
use deno_node_transform::TransformOutput;

use super::InMemoryLoader;

pub struct TestBuilder {
  loader: InMemoryLoader,
  entry_point: String,
  additional_entry_points: Vec<String>,
  test_entry_points: Vec<String>,
  specifier_mappings: HashMap<ModuleSpecifier, MappedSpecifier>,
  redirects: HashMap<ModuleSpecifier, ModuleSpecifier>,
  shims: Vec<Shim>,
  test_shims: Vec<Shim>,
}

impl TestBuilder {
  pub fn new() -> Self {
    let loader = InMemoryLoader::new();
    Self {
      loader,
      entry_point: "file:///mod.ts".to_string(),
      additional_entry_points: Vec::new(),
      test_entry_points: Vec::new(),
      specifier_mappings: Default::default(),
      redirects: Default::default(),
      shims: Default::default(),
      test_shims: Default::default(),
    }
  }

  pub fn with_loader(
    &mut self,
    mut action: impl FnMut(&mut InMemoryLoader),
  ) -> &mut Self {
    action(&mut self.loader);
    self
  }

  pub fn entry_point(&mut self, value: impl AsRef<str>) -> &mut Self {
    self.entry_point = value.as_ref().to_string();
    self
  }

  pub fn add_entry_point(&mut self, value: impl AsRef<str>) -> &mut Self {
    self
      .additional_entry_points
      .push(value.as_ref().to_string());
    self
  }

  pub fn add_test_entry_point(&mut self, value: impl AsRef<str>) -> &mut Self {
    self.test_entry_points.push(value.as_ref().to_string());
    self
  }

  pub fn add_default_shims(&mut self) -> &mut Self {
    let deno_shim = Shim {
      package: MappedSpecifier {
        name: "@deno/shim-deno".to_string(),
        version: Some("^0.1.0".to_string()),
        sub_path: None,
      },
      global_names: vec![GlobalName {
        name: "Deno".to_string(),
        export_name: None,
        type_only: false,
      }],
    };
    self.add_shim(deno_shim.clone());
    self.add_test_shim(deno_shim);
    let timers_shim = Shim {
      package: MappedSpecifier {
        name: "@deno/shim-timers".to_string(),
        version: Some("^0.1.0".to_string()),
        sub_path: None,
      },
      global_names: vec![
        GlobalName {
          name: "setTimeout".to_string(),
          export_name: None,
          type_only: false,
        },
        GlobalName {
          name: "setInterval".to_string(),
          export_name: None,
          type_only: false,
        },
      ],
    };
    self.add_shim(timers_shim.clone());
    self.add_test_shim(timers_shim);
    self
  }

  pub fn add_shim(&mut self, shim: Shim) -> &mut Self {
    self.shims.push(shim);
    self
  }

  pub fn add_test_shim(&mut self, shim: Shim) -> &mut Self {
    self.test_shims.push(shim);
    self
  }

  pub fn add_specifier_mapping(
    &mut self,
    specifier: impl AsRef<str>,
    bare_specifier: impl AsRef<str>,
    version: Option<&str>,
    path: Option<&str>,
  ) -> &mut Self {
    self.specifier_mappings.insert(
      ModuleSpecifier::parse(specifier.as_ref()).unwrap(),
      MappedSpecifier {
        name: bare_specifier.as_ref().to_string(),
        version: version.map(|v| v.to_string()),
        sub_path: path.map(|v| v.to_string()),
      },
    );
    self
  }

  pub fn add_redirect(
    &mut self,
    from: impl AsRef<str>,
    to: impl AsRef<str>,
  ) -> &mut Self {
    self.redirects.insert(
      ModuleSpecifier::parse(from.as_ref()).unwrap(),
      ModuleSpecifier::parse(to.as_ref()).unwrap(),
    );
    self
  }

  pub async fn transform(&self) -> Result<TransformOutput> {
    let mut entry_points =
      vec![ModuleSpecifier::parse(&self.entry_point).unwrap()];
    entry_points.extend(
      self
        .additional_entry_points
        .iter()
        .map(|p| ModuleSpecifier::parse(p).unwrap()),
    );
    transform(TransformOptions {
      entry_points,
      test_entry_points: self
        .test_entry_points
        .iter()
        .map(|p| ModuleSpecifier::parse(p).unwrap())
        .collect(),
      shims: self.shims.clone(),
      test_shims: self.test_shims.clone(),
      loader: Some(Box::new(self.loader.clone())),
      specifier_mappings: self.specifier_mappings.clone(),
      redirects: self.redirects.clone(),
    })
    .await
  }
}
