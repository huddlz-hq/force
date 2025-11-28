use crate::config::LoadedScript;
use crate::env::ForceEnv;
use std::process::Command;

/// Run a script with the force environment
pub fn run_script(
    script: &LoadedScript,
    env: &ForceEnv,
) -> Result<(), Box<dyn std::error::Error>> {
    let description = script
        .script
        .up
        .description
        .as_deref()
        .unwrap_or(&script.name);

    println!(
        "\n[{}/{}] {}",
        script.script.meta.category, script.name, description
    );

    let status = Command::new("sh")
        .arg("-c")
        .arg(&script.script.up.run)
        .envs(env.to_env_vars())
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(format!(
            "Script '{}' failed with exit code {}",
            script.name, code
        )
        .into());
    }

    Ok(())
}
