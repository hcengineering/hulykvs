use std::fs;
use std::path::PathBuf;
use crate::CONFIG;

pub async fn patch_uuid_in_migration() -> anyhow::Result<()> {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"),"etc","migrations","V2__workspace_uuid.sql"].iter().collect();

    let sql = fs::read_to_string(&path)?;

    let re = regex::Regex::new(r"UUID NOT NULL DEFAULT '([0-9a-fA-F-]{36})'")?;

    let patched_sql = re.replace_all(&sql, |caps: &regex::Captures| {
        if &caps[1] != CONFIG.default_workspace_uuid {
    	    caps[0].replace(&caps[1], &CONFIG.default_workspace_uuid)
        } else {
            caps[0].to_string()
        }
    });

    // Rewrite if needed
    if patched_sql != sql {
        fs::write(&path, patched_sql.as_ref())?;
	println!("File {} patched with uuid={}", path.display(), CONFIG.default_workspace_uuid );
    }

    Ok(())
}
