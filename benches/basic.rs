use codspeed_criterion_compat::{
  black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use std::{fs::read_to_string, path::Path};
use tree_shake::{tree_shake, TreeShakeConfig, TreeShakeOptions};

fn run_tree_shaker(source_text: String) -> String {
  let result = tree_shake(
    source_text,
    TreeShakeOptions {
      config: TreeShakeConfig::recommended(),
      minify_options: None,
      codegen_options: Default::default(),
    },
  );

  result.codegen_return.code
}

const FIXTURES: &[&str] = &["vue", "vuetify", "react"];

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
