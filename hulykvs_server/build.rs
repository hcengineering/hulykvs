use std::{env, fs, path::Path};
use regex::Regex;

fn main() {

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&manifest_dir);

    // UUID from TOML

    let config_path = root.join("src/config/default.toml");
    let text = fs::read_to_string(&config_path).expect("Cannot read default.toml");

    let uuid_line = text
	.lines()
	.find(|line| line.trim_start().starts_with("default_workspace_uuid"))
	.expect("default_workspace_uuid not found");

    let uuid = uuid_line
	.split('=')
	.nth(1)
        .expect("No = in line")
        .trim()
	.trim_matches('"')
        .to_string();

    let migration_path = root.join("etc/migrations/V2__workspace_uuid.sql");

    let original_sql = fs::read_to_string(&migration_path).expect("Cannot read migration");

    // UUID
    let re = Regex::new(
        r"UUID NOT NULL DEFAULT '([0-9a-fA-F-]{36})'",
    ).unwrap();

    let patched_sql = re.replace_all(&original_sql, |caps: &regex::Captures| {
        if &caps[1] != uuid {
            caps[0].replace(&caps[1], &uuid)
        } else {
            caps[0].to_string()
        }
    });

    if patched_sql != original_sql {
        fs::write(&migration_path, patched_sql.as_ref())
            .expect("Failed to write patched migration");
        println!("cargo:warning=Patched migration V2 with UUID: {}", uuid);
    }
}
