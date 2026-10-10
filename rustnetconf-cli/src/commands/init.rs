//! `netconf init` — create a project skeleton.

use std::path::Path;

pub fn run(project_dir: &Path) -> Result<(), String> {
    // Create directories
    let dirs = ["desired/example", ".netconf/state"];
    for dir in &dirs {
        let path = project_dir.join(dir);
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("failed to create {}: {e}", path.display()))?;
    }

    // Create a sample desired-state file so `desired/` isn't empty on first
    // run. Rename/replace `example/` with your own device name(s) — it is
    // not loaded by any command until you reference "example" as a device.
    let example_xml_path = project_dir.join("desired/example/interfaces.xml");
    if !example_xml_path.exists() {
        let example_xml = r#"<interfaces>
  <interface>
    <name>ge-0/0/0</name>
    <description>uplink</description>
  </interface>
</interfaces>
"#;
        std::fs::write(&example_xml_path, example_xml)
            .map_err(|e| format!("failed to write {}: {e}", example_xml_path.display()))?;
    }

    // Create inventory.toml if it doesn't exist
    let inventory_path = project_dir.join("inventory.toml");
    if !inventory_path.exists() {
        let template = r#"# rustnetconf inventory
# Define your network devices here.

[defaults]
confirm_timeout = 60
# username = "admin"

# [devices.spine-01]
# host = "10.0.0.1:830"
# username = "admin"
# key_file = "~/.ssh/id_ed25519"
# vendor = "junos"  # optional, auto-detected
#
# The SSH host key must be pinned before connecting (fail-closed by
# default). Pick one of the two options below — setting both is an error:
#
# Option A: pin the exact host key fingerprint. Read it on the device
# itself (console or an already-trusted session), not over the network:
#   Junos: file show /etc/ssh/ssh_host_ed25519_key.pub
#   then on your workstation: echo '<that line>' | ssh-keygen -lf -
#
# ssh-keyscan -p 830 10.0.0.1 | ssh-keygen -lf -  is a lab-only shortcut:
# it trusts whatever answers on the network, so verify the result out of
# band before relying on it.
# host_key_fingerprint = "SHA256:replace-with-real-fingerprint"
#
# Option B: verify against a known_hosts file instead:
# known_hosts_path = "~/.ssh/known_hosts"

# [devices.spine-02]
# host = "10.0.0.2:830"
# username = "admin"
# password = "secret"
"#;
        std::fs::write(&inventory_path, template)
            .map_err(|e| format!("failed to write inventory.toml: {e}"))?;
        eprintln!("Created inventory.toml");
    } else {
        eprintln!("inventory.toml already exists, skipping");
    }

    // Create .gitignore for state dir
    let gitignore_path = project_dir.join(".netconf").join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, "state/\n")
            .map_err(|e| format!("failed to write .gitignore: {e}"))?;
    }

    eprintln!("Project initialized. Next steps:");
    eprintln!("  1. Edit inventory.toml: uncomment a [devices.<name>] section and fill in");
    eprintln!("     host, username, and a host key pin (host_key_fingerprint or known_hosts_path)");
    eprintln!("  2. Replace desired/example/ with desired/<device-name>/ containing your XML");
    eprintln!("     config files (see desired/example/interfaces.xml for the shape)");
    eprintln!("  3. Run: netconf plan <device-name>");
    eprintln!();
    eprintln!("Note: plan/get/validate/apply all need a reachable NETCONF-capable device —");
    eprintln!("there is no offline or mock mode in this CLI. A lab device such as a vSRX");
    eprintln!("instance is the lightest way to try it end-to-end.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desired::read_desired_configs;
    use crate::inventory::Inventory;

    #[test]
    fn scaffolds_desired_example_with_valid_xml() {
        let tmp = tempfile::tempdir().unwrap();
        run(tmp.path()).unwrap();

        assert!(tmp.path().join("desired/example/interfaces.xml").exists());

        // The scaffolded file must be real, parseable desired-state XML —
        // not just present — so a first `netconf plan example` only fails
        // for lack of a device, never for malformed scaffolding.
        let configs = read_desired_configs(tmp.path(), "example").unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "interfaces");
    }

    #[test]
    fn inventory_template_has_devices_commented_out_and_fingerprint_example() {
        let tmp = tempfile::tempdir().unwrap();
        run(tmp.path()).unwrap();

        let template = std::fs::read_to_string(tmp.path().join("inventory.toml")).unwrap();
        assert!(
            template.contains("# host_key_fingerprint"),
            "template should show a commented host_key_fingerprint example"
        );
        assert!(
            template.contains("ssh-keygen"),
            "template should say how to obtain the fingerprint"
        );
        assert!(
            template.contains("-p 830"),
            "any ssh-keyscan suggestion must target the NETCONF port, not default SSH"
        );
        assert!(
            !template.contains("<("),
            "template must not rely on bash-only process substitution"
        );

        // Fresh template has no uncommented [devices.*] section, so loading
        // it must fail with the friendly "no devices" hint, not a panic or
        // a confusing parse error.
        let inventory_path = tmp.path().join("inventory.toml");
        let err = Inventory::load(&inventory_path).unwrap_err();
        assert!(err.contains("netconf init"));
    }

    #[test]
    fn rerunning_init_does_not_clobber_existing_files() {
        let tmp = tempfile::tempdir().unwrap();
        run(tmp.path()).unwrap();

        let custom = "<interfaces><interface><name>custom</name></interface></interfaces>\n";
        std::fs::write(tmp.path().join("desired/example/interfaces.xml"), custom).unwrap();
        std::fs::write(tmp.path().join("inventory.toml"), "custom-contents").unwrap();

        run(tmp.path()).unwrap();

        assert_eq!(
            std::fs::read_to_string(tmp.path().join("desired/example/interfaces.xml")).unwrap(),
            custom
        );
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("inventory.toml")).unwrap(),
            "custom-contents"
        );
    }
}
