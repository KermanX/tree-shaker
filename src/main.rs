use clap::Parser;
use oxc::{
  codegen::CodegenOptions,
  minifier::{MangleOptions, MinifierOptions},
};
use std::{fs::File, io::Write, path::PathBuf};
use tree_shake::{tree_shake, TreeShakeConfig, TreeShakeOptions};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  path: String,

  #[arg(short, long)]
  output: Option<String>,

  #[arg(short, long, default_value_t = false)]
  no_shake: bool,

  #[arg(short, long, default_value_t = false)]
  minify: bool,

  #[arg(short, long, default_value_t = String::from("recommended"))]
  preset: String,

  #[arg(short, long, default_value_t = false)]
  always_inline_literal: bool,

  #[arg(short, long, default_value_t = true)]
  jsx: bool,

  #[arg(long, default_value_t = false)]
  no_mangle: bool,
}

fn main() {
  let args = Args::parse();

  let source = match std::fs::read_to_string(&args.path) {
    Err(why) => {
      eprintln!("Couldn't read {}: {}", args.path, why);
      std::process::exit(1);
    }
    Ok(content) => content,
  };

  let start_time = std::time::Instant::now();

  let minify_options = MinifierOptions {
    mangle: Some(MangleOptions { top_level: true, ..Default::default() }),
    ..Default::default()
  };

  let copied = tree_shake(
    source.clone(),
    TreeShakeOptions {
      config: TreeShakeConfig::disabled().with_react_jsx(args.jsx),
      minify_options: None,
      codegen_options: CodegenOptions::default(),
    },
  );
  let minified = tree_shake(
    source.clone(),
    TreeShakeOptions {
      config: TreeShakeConfig::disabled().with_react_jsx(args.jsx),
      minify_options: Some(minify_options),
      codegen_options: CodegenOptions { minify: true, comments: false, ..Default::default() },
    },
  );
  let shaken = tree_shake(
    source.clone(),
    TreeShakeOptions {
      config: match args.preset.as_str() {
        "safest" => TreeShakeConfig::safest(),
        "recommended" => TreeShakeConfig::recommended(),
        "smallest" => TreeShakeConfig::smallest(),
        _ => {
          eprintln!("Invalid preset: {}", args.preset);
          std::process::exit(1);
        }
      }
      .with_react_jsx(args.jsx)
      .with_always_inline_literal(args.always_inline_literal)
      .with_mangling(!args.no_mangle),
      minify_options: None,
      codegen_options: CodegenOptions::default(),
    },
  );
  let shaken_minified = tree_shake(
    shaken.codegen_return.code.clone(),
    TreeShakeOptions {
      config: TreeShakeConfig::disabled().with_react_jsx(args.jsx),
      minify_options: Some(minify_options),
      codegen_options: CodegenOptions { minify: true, comments: false, ..Default::default() },
    },
  );

  let elapsed = start_time.elapsed();

  for diagnostic in shaken.diagnostics.iter() {
    eprintln!("{}", diagnostic);
  }

  eprintln!("Completed in {:?}", elapsed);
  eprintln!("Original: {}B", copied.codegen_return.code.len());
  eprintln!("Minified: {}B", minified.codegen_return.code.len());
  eprintln!("  Shaken: {}B", shaken.codegen_return.code.len());
  eprintln!("    Both: {}B", shaken_minified.codegen_return.code.len());
  eprintln!(
    "Minified/Both = {:.2}%",
    (shaken_minified.codegen_return.code.len() as f64 / minified.codegen_return.code.len() as f64)
      * 100.0
  );

  // If the input file is dir/a.js, the output file will be dir/a.out.js
  let output_path = args.output.map_or_else(
    || {
      let mut output_path = PathBuf::from(&args.path);
      if !args.no_shake {
        output_path.set_extension("out.js");
      }
      if args.minify {
        output_path.set_extension("min.js");
      }
      if args.no_shake && !args.minify {
        output_path.set_extension("copy.js");
      }
      output_path
    },
    PathBuf::from,
  );

  let mut output_file = match File::create(&output_path) {
    Err(why) => {
      eprintln!("Couldn't create {}: {}", output_path.display(), why);
      std::process::exit(1);
    }
    Ok(file) => file,
  };

  let code = match (!args.no_shake, args.minify) {
    (true, true) => shaken_minified.codegen_return.code,
    (true, false) => shaken.codegen_return.code,
    (false, true) => minified.codegen_return.code,
    (false, false) => copied.codegen_return.code,
  };
  match output_file.write_all(code.as_bytes()) {
    Err(why) => {
      eprintln!("Couldn't write to {}: {}", output_path.display(), why);
      std::process::exit(1);
    }
    Ok(_) => {
      println!("Wrote to {}", output_path.display());
    }
  }
}
