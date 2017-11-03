
use fail::{Failure, failure};
use std::process::Command;

/// Executes the zign command
pub fn zign() -> Result<String, Failure> {
    let output = Command::new("zign").arg("token").output().map_err(|err| failure("Failed to run zign command", err))?;
    if !output.status.success() {
        let exit_code = output.status.code().ok_or(failure("zign command was interrupted", ""))?;
        let stderr = String::from_utf8(output.stderr).map_err(|err| failure("Failed to decode zign stderr output as UTF-8", err))?;
        return Err(failure(&format!("zign command failed with exit code {}", exit_code), stderr))
    }
    String::from_utf8(output.stdout).map_err(|err| failure("Failed to decode zign stdout output as UTF-8", err))
}
