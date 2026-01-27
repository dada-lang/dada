use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Rfc {
    #[structopt(subcommand)]
    command: RfcCommand,
}

#[derive(StructOpt)]
pub enum RfcCommand {
    New {
        /// The name of the new RFC (e.g., "string-interpolation")
        name: String,
    },
}

impl Rfc {
    pub fn main(&self) -> anyhow::Result<()> {
        match &self.command {
            RfcCommand::New { name } => self.create_new_rfc(name),
        }
    }

    fn create_new_rfc(&self, name: &str) -> anyhow::Result<()> {
        let xtask_dir = cargo_path("CARGO_MANIFEST_DIR")?;
        let manifest_dir = xtask_dir.parent().unwrap().parent().unwrap();
        let rfcs_dir = manifest_dir.join("rfcs").join("src");

        // Find the next RFC number
        let rfc_number = self.find_next_rfc_number(&rfcs_dir)?;
        let rfc_dir_name = format!("{:04}-{}", rfc_number, name);
        let rfc_dir = rfcs_dir.join(&rfc_dir_name);

        // Create RFC directory
        fs::create_dir_all(&rfc_dir)?;

        // Copy template files
        self.copy_template_files(&rfcs_dir, &rfc_dir, rfc_number, name)?;

        // Update SUMMARY.md
        self.update_summary(&rfcs_dir, rfc_number, name, &rfc_dir_name)?;

        println!(
            "Created RFC-{:04}: {} in {}",
            rfc_number,
            name,
            rfc_dir.display()
        );
        println!("Don't forget to update the RFC status when ready!");

        Ok(())
    }

    fn find_next_rfc_number(&self, rfcs_dir: &PathBuf) -> anyhow::Result<u32> {
        let mut max_number = 0;

        for entry in fs::read_dir(rfcs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name();
                let dir_str = dir_name.to_string_lossy();

                // Extract number from directory names like "0001-feature-name"
                if let Some(number_str) = dir_str.split('-').next() {
                    if let Ok(number) = number_str.parse::<u32>() {
                        max_number = max_number.max(number);
                    }
                }
            }
        }

        Ok(max_number + 1)
    }

    fn copy_template_files(
        &self,
        rfcs_dir: &PathBuf,
        rfc_dir: &PathBuf,
        rfc_number: u32,
        name: &str,
    ) -> anyhow::Result<()> {
        let template_dir = rfcs_dir.join("0000-template");

        // Iterate through all files in the template directory
        for entry in fs::read_dir(&template_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let file_name = entry.file_name();
                let template_file = entry.path();
                let target_file = rfc_dir.join(&file_name);

                // Read template content
                let content = fs::read_to_string(&template_file)?;

                // Replace template placeholders
                let updated_content = self.process_template_content(&content, rfc_number, name);

                // Write to new RFC directory
                fs::write(&target_file, updated_content)?;
            }
        }

        Ok(())
    }

    fn process_template_content(&self, content: &str, rfc_number: u32, name: &str) -> String {
        let rfc_title = name.replace('-', " ");

        content
            .replace("RFC-0000", &format!("RFC-{:04}", rfc_number))
            .replace(
                "RFC-0000: Template",
                &format!("RFC-{:04}: {}", rfc_number, rfc_title),
            )
            .replace(
                "> **Note:** To create a new RFC, run `cargo xtask rfc new feature-name`\n\n",
                "",
            )
    }

    fn update_summary(
        &self,
        rfcs_dir: &PathBuf,
        rfc_number: u32,
        name: &str,
        rfc_dir_name: &str,
    ) -> anyhow::Result<()> {
        let summary_path = rfcs_dir.join("SUMMARY.md");
        let summary_content = fs::read_to_string(&summary_path)?;

        // Find the "# Active RFCs" section and add the new RFC
        let rfc_title = name.replace('-', " ");
        let new_rfc_line = format!(
            "- [RFC-{:04}: {}]({}/README.md)",
            rfc_number, rfc_title, rfc_dir_name
        );

        let updated_content = if let Some(_active_pos) = summary_content.find("# Active RFCs") {
            let mut lines: Vec<&str> = summary_content.lines().collect();

            // Find the line after "# Active RFCs"
            let mut insert_pos = None;
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with("# Active RFCs") {
                    insert_pos = Some(i + 2); // Skip the empty line after the header
                    break;
                }
            }

            if let Some(pos) = insert_pos {
                lines.insert(pos, &new_rfc_line);
                lines.join("\n")
            } else {
                summary_content
            }
        } else {
            summary_content
        };

        fs::write(summary_path, updated_content)?;
        Ok(())
    }
}

fn cargo_path(env_var: &str) -> anyhow::Result<PathBuf> {
    match std::env::var(env_var) {
        Ok(s) => {
            tracing::debug!("cargo_path({env_var}) = {s}");
            Ok(PathBuf::from(s))
        }
        Err(_) => anyhow::bail!("`{}` not set", env_var),
    }
}
