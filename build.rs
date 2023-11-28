
use std::fs::File;
use std::io::{BufWriter, Read};

fn main() -> anyhow::Result<()> {
    let config = p4d_mdproof::Config {
        // default_font: String::from("Arial"),

        ..Default::default()
    };


    let md = {
        let mut file = File::open("src/doc/SapConsumption.md")?;
        let mut md = String::new();
        file.read_to_string(&mut md)?;

        md
    };

    let file = File::create("deploy/SapConsumption.pdf")?;
    let pdf = p4d_mdproof::markdown_to_pdf(&md, &config)?;
    pdf.save(&mut BufWriter::new(file))?;

    Ok(())
}
