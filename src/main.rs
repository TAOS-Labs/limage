use limage::{args::{LimageArgs, LimageCommand}, builder::{self, Builder}, config::{self, Config}, runner};
use std::{env, process, path::Path};
use anyhow::{anyhow, Context, Result};

pub fn main() -> Result<()> {
    validate_executable_name()?;
    validate_arguments()?;

    let builder = Builder::new(None)?;
    let config = config::read_config(builder.manifest_path())?;
    process(&config)?;

    Ok(())
}

fn validate_executable_name() -> Result<()> {
    let executable_name = env::args()
        .next()
        .ok_or_else(|| anyhow!("no first argument (executable name)"))?;
    let file_stem = Path::new(&executable_name)
        .file_stem()
        .and_then(|s| s.to_str());
    if file_stem != Some("limage") {
        return Err(anyhow!(
            "Unexpected executable name: expected `limage`, got: `{:?}`",
            file_stem
        ));
    }

    Ok(())
}

fn validate_arguments() -> Result<()> {
    let mut raw_args = env::args();
    raw_args.next(); // Skip executable name

    match raw_args.next().as_deref() {
        Some("clean") | Some("build") | Some("runner") | None => (),
        Some("--help") | Some("-h") => {
            println!("Limage is a tool for building and running kernels.");
            println!();
            println!("USAGE:");
            println!("    limage [OPTIONS] <SUBCOMMAND> [ARGS]");
            println!();
            println!("OPTIONS:");
            println!("    -h, --help       Prints help information");
            println!("    -v, --version    Prints version information");
            println!();
            println!("SUBCOMMANDS:");
            println!("    build            Builds the kernel");
            println!("    runner           Runs the kernel");
            return Ok(());
        },
        Some("--version") | Some("-v") => {
            println!("limage v0.5.1");
            return Ok(());
        },
        Some(arg) => return Err(anyhow!(
            "Unexpected subcommand: `{}`. See `limage --help` for more information.",
            arg
        ))
    }

    Ok(())
}

fn process(config: &Config) -> Result<()> {
    let raw_args = env::args();
    let exit_code = match LimageCommand::parse_args(raw_args)? {
        LimageCommand::Build => Some(builder::build(&config)?),
        LimageCommand::Run(args) => Some(runner(args)?),
        LimageCommand::Version => None,
        LimageCommand::Help => None
    };

    if let Some(code) = exit_code {
        process::exit(code);
    }

    Ok(())
}

pub(crate) fn runner(args: LimageArgs) -> Result<i32> {
    let mut builder = Builder::new(None)?;
    let config = config::read_config(builder.manifest_path())?;

    let exe_parent = args
        .executable
        .parent()
        .ok_or_else(|| anyhow!("kernel executable has no parent"))?;
    let is_doctest = exe_parent
        .file_name()
        .ok_or_else(|| anyhow!("kernel executable's parent has no file name"))?
        .to_str()
        .ok_or_else(|| anyhow!("kernel executable's parent file name is not valid UTF-8"))?
        .starts_with("rustdoctest");
    let is_test = is_doctest || exe_parent.ends_with("deps");

    let executable_canonicalized = args.executable.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize executable path `{}`",
            args.executable.display(),
        )
    })?;

    builder.build(&config, &Some(executable_canonicalized))?;
    
    let exit_code = runner::run(config, is_test)?;

    process::exit(exit_code);
}