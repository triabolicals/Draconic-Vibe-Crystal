use std::sync::OnceLock;
use engage::{Dictionary_2Ext, app::ResourceManager_2, };
use crate::utils::{il2str};
pub static SEARCH_LIST: OnceLock<OtherAssetData> = OnceLock::new();

pub struct OtherAssetData {
    pub map_events: Vec<String>,
    pub reliance: Vec<String>,
}
impl OtherAssetData {
    pub fn new() -> Self {
        let files = ResourceManager_2::get_s_files();
        let mut reliance = vec![];
        let mut map_events = vec![];
        files.iter().filter_map(|s| il2str(s.0).filter(|v| v.contains("Event/PuppetDemo/Materials/") || v.contains("Puppet/R_")))  // || v.contains("Puppet/GR_")))
            .for_each(|v| {
                let file = v.split("/").last().unwrap().split_once(".").unwrap().0.to_string();
                if file.starts_with("R_") { reliance.push(file); }
                else { map_events.push(file); }
            });

        Self{ reliance, map_events }
    }
    pub fn init() { SEARCH_LIST.get_or_init(|| Self::new()); }
    pub fn get() -> &'static Self { SEARCH_LIST.get_or_init(|| Self::new()) }
}