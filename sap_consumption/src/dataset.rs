// tokio has asyncronous file operations,
//  but we don't need to use them since file
//  operations are done on the main thread.
use std::fs::File;
use std::io::Write;

use std::path::PathBuf;

// use tiberius::Result;
use sysinteg_db::{DbClient, DbResult};


pub enum Dataset {
    Production,
    Issue
}

impl Dataset {
    fn name(&self) -> &str {
        match self {
            Self::Production => "Production",
            Self::Issue => "Issue"
        }
    }

    fn query(&self) -> &str {
        match self {
            Self::Production => "EXEC SapProductionData_SinceLastRun @End = @P1",
            Self::Issue      => "EXEC SapIssueData_SinceLastRun @End = @P1",
        }
    }

    fn filename(&self, end: chrono::NaiveDateTime, output_dir: &PathBuf) -> PathBuf {
        let filename = format!("{}_{}.ready", self.name(), end.format("%Y%m%d%H%M%S"));
        let filename = output_dir.join(filename);

        filename
    }

    pub async fn pull_data(self, client: &mut DbClient, end: chrono::NaiveDateTime, output_dir: &PathBuf) -> DbResult<()> {
        let name = self.name();

        log::trace!("pulling dataset `{}`", name);
        let data = client.query(self.query(), &[&end]).await?
            .into_first_result().await?;

        if data.len() == 0 {
            log::info!("Dataset `{}` is empty", name);
        } else {
            // TODO: store on server for verification once feedback loop from SAP is established
            let filename = self.filename(end, output_dir);
            let mut file = File::create(&filename)
                .map_err(|error| {
                    log::error!("Failed to create {} file {}", name, &filename.to_str().unwrap());
                
                    error
                })?;
        
            log::trace!("Writing dataset `{}`", name);
            let file_contents = data
                .into_iter()
                // convert row to tab delimited string
                .map(|row| sysinteg_db::row_to_string(row))
                .collect::<Vec<String>>()
                .join("\n");
    
            file.write_all(file_contents.as_bytes())
                .map_err(|error| {
                    log::error!("failed to write dataset `{}` to {}. Deleting file.", name, &filename.to_str().unwrap());
                    let _ = std::fs::remove_file(filename);

                    error
                })?;
        }

        // update last runtime 
        client.execute("UPDATE HighSteel.RuntimeInfo SET last_runtime=@P1 WHERE name=@P2", &[&end, &format!("Sap{}Data", name)]).await?;
            
        Ok(())
    }
}
