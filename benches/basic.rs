use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
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

pub fn criterion_benchmark(c: &mut Criterion) {
  let input_path = Path::new("./benches/fixtures/vue-starter.js");
  let source_text = match std::fs::read_to_string(&input_path) {
    Err(why) => {
      eprintln!("Couldn't read {}: {}", input_path.display(), why);
      std::process::exit(1);
    }
    Ok(content) => content,
  };

  c.bench_function("vue-starter", |b| b.iter(|| run_tree_shaker(black_box(source_text.clone()))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
