use clap::Parser;
use oxc::{codegen::CodegenOptions, minifier::MinifierOptions};
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

  #[arg(short, long, default_value_t = false)]
  logging: bool,
}

fn main() {
  let args = Args::parse();

  let content = match std::fs::read_to_string(&args.path) {
    Err(why) => {
      eprintln!("Couldn't read {}: {}", args.path, why);
      std::process::exit(1);
    }
    Ok(content) => content,
  };

  let start_time = std::time::Instant::now();

  let result = tree_shake(
    content,
    TreeShakeOptions {
      config: if args.no_shake {
        TreeShakeConfig::disabled()
      } else {
        TreeShakeConfig::recommended()
      },
      minify_options: args.minify.then(MinifierOptions::default),
      codegen_options: CodegenOptions {
        minify: args.minify,
        comments: !args.minify,
        ..Default::default()
      },
      logging: args.logging,
    },
  );

  let elapsed = start_time.elapsed();

  for diagnostic in result.diagnostics.iter() {
    eprintln!("{}", diagnostic);
  }

  eprintln!("[tree-shaker] Finished in {:?}", elapsed);

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
        output_path.set_extension("out.js");
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

  match output_file.write_all(result.codegen_return.code.as_bytes()) {
    Err(why) => {
      eprintln!("Couldn't write to {}: {}", output_path.display(), why);
      std::process::exit(1);
    }
    Ok(_) => {
      println!("Wrote to {}", output_path.display());
    }
  }
}
