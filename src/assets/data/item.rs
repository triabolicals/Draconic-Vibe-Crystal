use engage::{
    gamedata::item::ItemData, random::Random
};

pub struct ItemAsset {
    pub iid_index: i32,
    pub asset_entry: i32,
    pub kind: u8,
}

impl ItemAsset {
    pub fn new(item: &ItemData, entry_index: i32) -> Self {
        Self {
            iid_index: item.parent.index, 
            asset_entry: entry_index,
            kind: item.kind as u8,
        }
    }
}

pub struct WeaponAssets {
    pub bows: Vec<ItemAsset>,
    pub tome: Vec<ItemAsset>,
    pub melee: Vec<ItemAsset>,
    pub rods: Vec<ItemAsset>,
}

impl WeaponAssets {
    pub fn new() -> Self { Self { bows: Vec::new(), tome: Vec::new(), melee: Vec::new(), rods: Vec::new() } }
    pub fn add(&mut self, item: &ItemData, entry_index: i32) {
        let new = ItemAsset::new(item, entry_index);
        match new.kind {
            4 => { self.bows.push(new); }
            6 => { self.tome.push(new); }
            7 => { self.rods.push(new); }
            1|2|3|5 => { self.melee.push(new); }
            _ => {}
        }
    }
    pub fn get_random(&self, kind: u32, rng: &Random) -> &ItemAsset {
        match kind {
            6 => {
                match rng.get_value(6) {
                    0|1|2 => { &self.melee[ rng.get_value( self.melee.len() as i32 ) as usize ] }
                    3|4 => { &self.tome[ rng.get_value( self.tome.len() as i32 ) as usize ] }
                    _ =>  { &self.rods [ rng.get_value( self.rods.len() as i32 ) as usize ] }
                }
            }
            7 => { &self.rods [ rng.get_value( self.rods.len() as i32 ) as usize ] }
            4 => { &self.bows [ rng.get_value( self.bows.len() as i32 ) as usize ] }
            _ => {
                match rng.get_value(3) {
                    0|1 => { &self.melee[ rng.get_value( self.melee.len() as i32 ) as usize ] }
                    _ => { &self.rods [ rng.get_value( self.rods.len() as i32 ) as usize ] }
                }
            }
        }
    }
    pub fn get_index(&self, index: i32) -> Option<&ItemAsset> {
        if let Some(asset) = self.melee.iter().find(|w| w.iid_index == index) { Some(asset) }
        else if let Some(asset) = self.tome.iter().find(|w| w.iid_index == index) { Some(asset) }
        else if let Some(asset) = self.bows.iter().find(|w| w.iid_index == index) { Some(asset) }
        else if let Some(asset) = self.rods.iter().find(|w| w.iid_index == index) { Some(asset) }
        else { None }
    }
}