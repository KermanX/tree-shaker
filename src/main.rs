use std::{env, fs::File, io::Write, path::Path};
use tree_shake::{tree_shake, TreeShakeConfig, TreeShakeOptions};

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    eprintln!("Usage: {} <filename>", args[0]);
    std::process::exit(1);
  }

  let path = Path::new(&args[1]);
  let content = match std::fs::read_to_string(&path) {
    Err(why) => {
      eprintln!("Couldn't read {}: {}", path.display(), why);
      std::process::exit(1);
    }
    Ok(content) => content,
  };

  let start_time = std::time::Instant::now();

  let allocator = Default::default();
  let result = tree_shake(TreeShakeOptions {
    config: TreeShakeConfig::recommended(),
    allocator: &allocator,
    source_type: Default::default(),
    source_text: content,
    tree_shake: true,
    minify: None,
    code_gen: Default::default(),
    eval_mode: false,
    logging: false,
  });

  let elapsed = start_time.elapsed();

  for diagnostic in result.diagnostics.iter() {
    eprintln!("{}", diagnostic);
  }

  eprintln!("[tree-shaker] Finished in {:?}", elapsed);

  // If the input file is dir/a.js, the output file will be dir/a.out.js
  let mut output_path = path.to_path_buf();
  output_path.set_extension("out.js");

  let mut output_file = match File::create(&output_path) {
    Err(why) => {
      eprintln!("Couldn't create {}: {}", output_path.display(), why);
      std::process::exit(1);
    }
    Ok(file) => file,
  };

  match output_file.write_all(result.codegen_return.source_text.as_bytes()) {
    Err(why) => {
      eprintln!("Couldn't write to {}: {}", output_path.display(), why);
      std::process::exit(1);
    }
    Ok(_) => {
      println!("Wrote to {}", output_path.display());
    }
  }
}
