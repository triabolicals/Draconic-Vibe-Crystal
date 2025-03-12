use unity::prelude::*;
use engage::{
    mess::Mess,
    random::Random,
    gamevariable::GameVariableManager,
    gamedata::{PersonData, Gamedata, assettable::AssetTableResult, unit::Unit},
};
use crate::config::DVCVariables;
use super::super::accessory::change_accessory;

use concat_string::concat_string;
pub struct HeadData {
    pub male_head: Vec<u16>,
    pub female_head: Vec<u16>,
    pub hair_m: Vec<(u16, bool)>,
    pub hair_f: Vec<(u16, bool)>,
    pub acc_list: Vec<(u16, i32, String)>,
    pub skin: Vec<SkinData>,
    pub aoc_m: Vec<String>,
    pub aoc_f: Vec<String>,
}

impl HeadData {
    pub fn reset_head_list(&mut self) {}
    pub fn add(&mut self, line: String){
        let values: Vec<_> = line.split_whitespace().collect();
        if values[0].contains("body"){
            let id = values[1].parse::<u16>().unwrap();
            if let Some(found) = self.skin.iter_mut().find(|s| s.id == id ) {
                found.is_unique = true;
                found.body = values[2].to_string();
                found.voice = values[3].to_string();
                if values.len() > 4 { found.rig = values[4].to_string(); }
            }
            else {
                let mut skin = SkinData::new();
                skin.id = id;
                skin.is_unique = true;
                skin.body = values[2].to_string();
                skin.voice = values[3].to_string();
                if values.len() > 4 { skin.rig = values[4].to_string(); }
                self.skin.push(skin);
            }
        }
        else if values[0].contains("acc") && values.len() >= 4 {
            if !values[3].is_empty() {
                let id = values[1].parse::<u16>().unwrap();
                match values[2] {
                    "h" => { self.acc_list.push( (id, 0, values[3].to_string() ) ); }
                    "s2" => { self.acc_list.push( (id, 1, values[3].to_string() ) ); }
                    "h2" => { self.acc_list.push( (id, 2, values[3].to_string() ) ); }
                    _ => { return; }
                }
            }
        }
        else if values[0].contains("aoc") {
            if values[0].contains("aoc_m") { for x in 1..values.len() { self.aoc_m.push(values[x].to_string());} }
            else { for x in 1..values.len() { self.aoc_f.push(values[x].to_string()); } }
        }
        else {
            let id = values[0].parse::<u16>().unwrap();
            let is_acc = values[1].parse::<u16>().unwrap() == 1;
            let hair_id = values[2].parse::<u16>().unwrap();
    
            if id % 100 >= 50 || id == 303 { // Females are cx50+ and Rosado
                if !self.female_head.iter().any(|&f| f == id) { self.female_head.push(id); }         
            } 
            else {  // Males for cx00 to cx49
                if !self.male_head.iter().any(|&m| m == id) { self.male_head.push(id); }
            }
            if hair_id % 100 < 50 {
                if !self.hair_m.iter().any(|&h| h.0 == hair_id) { self.hair_m.push( (hair_id, is_acc) );  } 
            }
            else {
                if !self.hair_f.iter().any(|&h| h.0 == hair_id) { self.hair_f.push( (hair_id, is_acc) );  } 
            }

            if values.len() == 17 && !self.skin.iter().any(|s| s.id == id) {
                let mut skin = SkinData::new();
                skin.id = id;
                for x in 0..14 { skin.scale[x] = values[3+x].parse::<f32>().unwrap();  }
                self.skin.push( skin );
            }
            else if values.len() >= 20 && !self.skin.iter().any(|s| s.id == id) {
                let mut skin = SkinData::new();
                skin.id = id;
                skin.r =  values[3].parse::<u8>().unwrap();
                skin.g =  values[4].parse::<u8>().unwrap();
                skin.b =  values[5].parse::<u8>().unwrap();
                for x in 0..14 { skin.scale[x] = values[6+x].parse::<f32>().unwrap();  }
                self.skin.push( skin );
            }
        }

    }
    pub fn replace_by_rng(&self, unit: &Unit, result: &mut AssetTableResult) {
        if unit.person.gender > 2 || unit.person.gender == 0 { return; }
        if GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) & 1 == 0 { return; }
        let rng = Random::instantiate().unwrap();
        rng.ctor( unit.drop_seed as u32 );
        let male = super::super::unit_dress_gender(unit) == 1;
        let head = 
            if male { self.male_head[ rng.get_value( self.male_head.len() as i32 ) as usize] }
            else { self.female_head[ rng.get_value( self.female_head.len() as i32 ) as usize] };
        
        let hair = 
        if head >= 800 || head == 504 {
            if male { &self.hair_m[ rng.get_value( self.hair_m.len() as i32) as usize ] }
            else {  &self.hair_f[ rng.get_value( self.hair_f.len() as i32) as usize ]  }
        }
        else if let Some(hair_m) = self.hair_m.iter().find(|x| x.0 == head) {  hair_m }
        else if let Some(hair_f) = self.hair_f.iter().find(|x| x.0 == head) { hair_f }
        else if male { &self.hair_m[ rng.get_value( self.hair_m.len() as i32) as usize ] }
        else { &self.hair_f[ rng.get_value( self.hair_f.len() as i32) as usize ] };
        if head == 302 { return; }

        let head_str = Self::pad_zeros(head);
        result.head_model = concat_string!("uHead_c", head_str).into();
        let hair_number = Self::pad_zeros(hair.0);
        if hair.1 {
            let model = concat_string!("uAcc_spine2_Hair", hair_number);
            result.hair_model = "uHair_null".into();
            change_accessory(result.accessory_list, model.as_str(), "c_spine1_jnt");
        }
        else {
            let hair = concat_string!("uHair_h", hair_number);
            result.hair_model = hair.into();
        }
        if let Some(skin) = self.skin.iter().find(|s| s.id == head ) {
            if skin.r != 0 && skin.g != 0 && skin.b != 0 {
                result.unity_colors[2].r = skin.r as f32 / 255.0;
                result.unity_colors[2].g = skin.g as f32 / 255.0; 
                result.unity_colors[2].b  = skin.g as f32 / 255.0;
            }
            for x in 0..9 {  if skin.scale[x] > 0.0 { result.scale_stuff[x] = skin.scale[x]; } }
            for x in 9..12 { if skin.scale[x+2] > 0.0 { result.scale_stuff[x] = skin.scale[x+2]; } }
            if skin.scale[9] > 0.0 { result.scale_stuff[12] = skin.scale[9]; }
            if skin.scale[10] > 0.0 { result.scale_stuff[13] = skin.scale[10]; }

            if skin.is_unique { // AOC_Inf
                // let number = if skin.voice == "_Blank" { head - 7 } else { head };
                if result.sound.voice.is_some() {
                    if !skin.voice.is_empty() { result.sound.voice = Some ( skin.voice.clone().into() ); }
                }
                if skin.body.len() > 2 && !GameVariableManager::get_bool("G_EnemyOutfits") {
                    let body = concat_string!("uBody_", skin.body, "_c", head_str);
                    // println!("Body Model: {} replacing {}", body, result.dress_model);
                    result.dress_model = body.into(); 
                }

                if skin.rig.len() > 3 { result.body_model = concat_string!("uRig_", skin.rig).into(); }    //Rig
                //println!("Rig: {}", result.body_model);
                change_accessory(result.accessory_list, "null", "c_spine2_jnt");
                change_accessory(result.accessory_list, "null", "c_head_loc");
            }
        }
        self.acc_list.iter().filter(|acc| acc.0 == head && !acc.2.is_empty() )
            .for_each(|acc|{
                if acc.1 == 0 || acc.1 == 2  {
                    let locator = if acc.1 == 0 { "c_head_loc" } else { "c_head2_loc"};
                    let model = concat_string!("uAcc_head_", acc.2, head_str);
                    super::super::accessory::change_accessory(result.accessory_list, model.as_str(), locator);
                }
            }
        );
    }
    fn pad_zeros(number: u16) -> String {
        if number < 10 { format!("00{}", number ) }
        else if number  < 100 { format!("0{}", number ) }
        else { format!("{}", number )}
    }
    pub fn random_aoc(&self, unit: &Unit, result: &mut AssetTableResult) {
        let hash = if unit.person.get_asset_force() == 0 { unit.person.parent.hash } else { unit.grow_seed };
        let rng = crate::utils::create_rng(hash, 1);
        if unit.status.value & 8388608 != 0 { rng.get_value(100); }
        let aoc = if super::super::unit_dress_gender(unit) == 1 { &self.aoc_m[ rng.get_value( self.aoc_m.len() as i32 ) as usize] }
            else { &self.aoc_f[ rng.get_value( self.aoc_f.len() as i32 ) as usize] };

        result.info_anims = Some(concat_string!("AOC_Info_c", aoc).into());

    }
    pub fn unique_head_for_generic(&self, result: &mut AssetTableResult, job_hash: i32, is_female: bool) {
        if GameVariableManager::get_number(DVCVariables::GENERIC_APPEARANCE_KEY) & 1 == 1 { return; }
        let rng = Random::instantiate().unwrap();
        rng.ctor( job_hash as u32 );
        let male = !is_female;
        let head = 
            if male {
                self.male_head[ rng.get_value( self.male_head.len() as i32 ) as usize]
            }
            else { 
                self.female_head[ rng.get_value( self.female_head.len() as i32 ) as usize]
            };
        
        let hair = 
            if male { &self.hair_m[ rng.get_value( self.hair_m.len() as i32 ) as usize ] }
            else {
                &self.hair_f[ rng.get_value( self.hair_f.len() as i32 ) as usize ] 
            };

        let head_str = Self::pad_zeros(head);
        result.head_model = concat_string!("uHead_c", head_str).into();
        let hair_number = Self::pad_zeros(hair.0);
        if hair.1 {
            let model = concat_string!("uAcc_spine2_Hair", hair_number);
            result.hair_model = "uHair_null".into();
            change_accessory(result.accessory_list, model.as_str(), "c_spine1_jnt");
        }
        else {
            let hair = concat_string!("uHair_h", hair_number);
            result.hair_model = hair.into();
        }
    }
}


pub struct SkinData {
    pub id: u16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub scale: [f32; 14],
    pub is_unique: bool,
    pub body: String,
    pub voice: String,
    pub rig: String,
}

impl SkinData {
    pub fn new() -> Self {
        Self {
            id: 0, r: 0, g: 0, b: 0,
            scale: [0.0; 14],
            is_unique: false,
            body: String::new(),
            voice: String::new(),
            rig: String::new(),
        }
    }
}

pub struct NameData {
    pub female: Vec<i16>,
    pub male: Vec<i16>,
    pub act_replace: Vec<(String, String, i32)>, 
}

impl NameData {
    pub fn get_len(&self) -> usize { return self.female.len() + self.male.len(); }
    pub fn add(&mut self, line: String) {
        let values: Vec<_> = line.split_whitespace().collect();
        let pid = values[0].to_string();

        if let Some(person) = PersonData::get(pid.as_str()) {
            let person_index = person.parent.index as i16;
            if person.gender == 1 { 
                if !self.male.iter().any(|&x| x == person_index) { self.male.push(person_index); }
            }
            else if person.gender == 2 { 
                if !self.female.iter().any(|&x| x == person_index) { self.female.push(person_index); }
            }
            else { return; }
            let mut counter = 1 as usize;
            loop {
                if counter >= values.len() { break; }
                if counter + 1 > values.len() { break; }
                let act = values[counter].to_string();
                let weapon = values[counter + 1].parse::<i32>().unwrap();
                let mpid = person.get_name().unwrap().to_string();
                self.act_replace.push( (mpid.clone(), act.clone(), weapon));
                counter += 2;
            }
        }
    } 
    pub fn get_random_person(&self, female: bool) -> &'static PersonData {
        let rng = Random::get_game();
        let index = 
            if female { self.female[ rng.get_value( self.female.len() as i32 ) as usize ] }
            else { self.male[rng.get_value(  self.male.len() as i32 ) as usize ]};
        return PersonData::try_index_get(index as i32).unwrap();
    }
}

pub fn get_names_data() -> NameData {
    let mut data = NameData{female: Vec::new(), male: Vec::new(), act_replace: Vec::new() };
    include_str!("data/names.txt").lines().into_iter().
        for_each(|line|{
            let new_line = line.to_string();
            data.add(new_line);
        }
    );
    println!("Initialized unique name data");
    data
}

pub fn get_head_data() -> HeadData {
    let mut head_stuff = HeadData{male_head: Vec::new(), female_head: Vec::new(), hair_m: Vec::new(), hair_f: Vec::new(),  acc_list: Vec::new(), skin: Vec::new(), aoc_f: Vec::new(), aoc_m: Vec::new() };
    include_str!("data/heads.txt").lines().into_iter()
        .for_each(|line|{
            let head_data = line.to_string();
            head_stuff.add(head_data);
        }
    );
    head_stuff.male_head.push(504);
    println!("Initialized head/skin data");
    head_stuff
}