use std::{env, fs, path::Path, process::Command};

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=package.json");
  println!("cargo:rerun-if-changed=package-lock.json");
  println!("cargo:rerun-if-changed=tsconfig.json");

  let ts_dir = Path::new("src/scripts");
  if ts_dir.exists() {
    rerun_if_changed(ts_dir);
  }

  fs::create_dir_all("src/gen").expect("failed to create generated JavaScript directory");

  if !Path::new("node_modules").exists() {
    run_npm(&["install"]);
  }

  // Type-checks with tsc, then bundles `src/scripts/prelude.ts` into `src/gen/prelude.js` with esbuild.
  run_npm(&["run", "build"]);
}

fn run_npm(args: &[&str]) {
  let npm = env::var_os("NPM");

  let mut command = if let Some(npm) = npm {
    Command::new(npm)
  } else if cfg!(target_os = "windows") {
    // Windows needs `cmd /C` to invoke npm batch files (npm.cmd)
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "npm"]);
    cmd
  } else {
    Command::new("npm")
  };

  command.args(args);

  let output = command
    .output()
    .unwrap_or_else(|err| panic!("failed to execute `npm`: {err}. Is `npm` in your PATH?"));

  if !output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    panic!("`npm {}` failed:\n{stdout}{stderr}", args.join(" "));
  }
}

fn rerun_if_changed(path: &Path) {
  println!("cargo:rerun-if-changed={}", path.display());

  if path.is_dir() {
    if let Ok(entries) = fs::read_dir(path) {
      for entry in entries.flatten() {
        rerun_if_changed(&entry.path());
      }
    }
  }
}
