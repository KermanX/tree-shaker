use codspeed_criterion_compat::{
  black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use std::{fs::read_to_string, path::Path};
use tree_shake::{tree_shake, TreeShakeConfig, TreeShakeOptions};

fn run_tree_shaker(source_text: String) -> String {
  let allocator = Default::default();
  let result = tree_shake(TreeShakeOptions {
    config: TreeShakeConfig::recommended(),
    allocator: &allocator,
    source_type: Default::default(),
    source_text,
    tree_shake: true,
    minify: None,
    code_gen: Default::default(),
    eval_mode: false,
    logging: false,
  });

  result.codegen_return.source_text
}

const FIXTURES: &[&str] = &[
  "vue",
  // "vuetify",
];

pub fn criterion_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("fixtures");

  for fixture in FIXTURES {
    let input_path = format!("./test/e2e/{fixture}/dist/bundled.js");
    let input_path = Path::new(&input_path);
    let source_text = read_to_string(&input_path).unwrap();

    group.bench_with_input(BenchmarkId::from_parameter(fixture), &source_text, |b, source_text| {
      b.iter(|| run_tree_shaker(black_box(source_text.clone())))
    });
  }

  group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
