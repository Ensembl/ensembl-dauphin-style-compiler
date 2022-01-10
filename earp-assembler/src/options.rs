use peregrine_cli_toolkit::{ config_from_options, ConfigOption };

const APP_NAME : &str = "earp assembler";
const APP_VERSION : &str = "0.0.0";
const APP_AUTHOR : &str = "Ensembl Webteam <ensembl-webteam@ebi.ac.uk>";
const APP_ABOUT : &str = "Assembler for earp source files into earp binary files: .earps -> .earp";

pub(crate) struct Config {
    pub source_files: Vec<String>,
    pub no_default_maps: bool,
    pub additional_maps: Vec<String>,
    pub object_file: String,
    pub verbose: u32
}

pub(crate) fn parse_config() -> Result<Config,String> {
    let mut config = Config {
        source_files: vec![],
        no_default_maps: false,
        additional_maps: vec![],
        object_file: "out.earp".to_string(),
        verbose: 0
    };
    config_from_options::<_,String>(&mut config,vec![
        ConfigOption::new("source file","source",Some("s"),Some("source.searp"),true,|c: &mut Config, v| {
            c.source_files.push(v.to_string());
            Ok(())
        }),
        ConfigOption::new("no default map files","no-default-maps",None,None,false,|c: &mut Config,_| {
            c.no_default_maps = true;
            Ok(())
        }),
        ConfigOption::new("additional map file","map",Some("M"),Some("opcodes.map"),true,|c: &mut Config, v| {
            c.additional_maps.push(v.to_string());
            Ok(())
        }),
        ConfigOption::new("object file","object",Some("o"),Some("out.earp"),false,|c: &mut Config, v| {
            c.object_file = v.to_string();
            Ok(())
        }),
        ConfigOption::new("verbose","verbose",Some("v"),None,true,|c: &mut Config,v| {
            c.verbose += u32::from_str_radix(v,10).ok().unwrap();
            Ok(())
        }),
        ],APP_NAME,APP_VERSION,APP_AUTHOR,APP_ABOUT)?;
    Ok(config)
}
