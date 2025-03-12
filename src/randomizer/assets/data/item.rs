use engage::{
    random::Random,
    gamedata::{Gamedata, item::ItemData},
};

pub struct WeaponAsset {
    pub iid_index: i32,
    pub right_hand: String,
    pub left_hand: String,
    pub kind: u8,
}

impl WeaponAsset {
    pub fn new(line: String) -> Self {
        let values: Vec<_> = line.split_whitespace().collect();
        let right_hand = values[1];
        let left_hand = if values.len() == 3 { values[2] } else { "none" };
        let index = ItemData::get(values[0]).unwrap().parent.index;
        let kind = ItemData::get(values[0]).unwrap().kind;
        Self {
            iid_index: index, 
            kind: kind as u8,
            right_hand: right_hand.to_string(),
            left_hand: left_hand.to_string()
        }
    }
}

pub struct WeaponAssets {
    pub bows: Vec<WeaponAsset>,
    pub tome: Vec<WeaponAsset>,
    pub melee: Vec<WeaponAsset>,
    pub rods: Vec<WeaponAsset>,
}

impl WeaponAssets {
    pub fn add(&mut self, data_line: String) {
        let new = WeaponAsset::new(data_line);
        match new.kind {
            4 => { self.bows.push(new); }
            6 => { self.tome.push(new); }
            7 => { self.rods.push(new); }
            1|2|3|5 => { self.melee.push(new); }
            _ => {}
        }
    }
    pub fn get_random(&self, kind: u32, rng: &Random) -> &WeaponAsset {
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
    pub fn get_index(&self, index: i32) -> Option<&WeaponAsset> {
        if let Some(asset) = self.melee.iter().find(|w| w.iid_index == index) { Some(asset) }
        else if let Some(asset) = self.tome.iter().find(|w| w.iid_index == index) { Some(asset) }
        else if let Some(asset) = self.bows.iter().find(|w| w.iid_index == index) { Some(asset) }
        else if let Some(asset) = self.rods.iter().find(|w| w.iid_index == index) { Some(asset) }
        else { None }
    }
}

pub fn get_weapon_assets() -> WeaponAssets {
    let mut weapons = WeaponAssets{ bows: Vec::new(), tome: Vec::new(), melee: Vec::new(), rods: Vec::new() };
    include_str!("data/items.txt").lines().into_iter()
        .for_each(|line|{
            let new_line = line.to_string();
            weapons.add(new_line);
        }
    );
    println!("Initialized weapon asset data");
    weapons
}