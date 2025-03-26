use super::*;



pub struct AnimData {
    pub act_name: String,
    pub gender: Gender,
    pub mount: Mount,
    pub weapon_mask: i32,
    pub has_transformation: bool,
    pub is_generic: bool,
    pub has_morph: bool,
    pub special: i32,
    pub suffix: String,
}
/*
pub struct ActData {
    pub anims: Vec<AnimData>,
    pub base_mount: BaseAnim,
}

pub struct BaseAnim {
    pub inf0a: [i32; 20],
    pub cav0b: [i32; 20],
    pub cav2c: [i32; 20],
    pub wng2d: [i32; 20],
    pub wng0e: [i32; 20],
    pub wng1f: [i32; 20],
}
impl BaseAnim {
    pub fn set(&mut self, index: i32, set: usize, kind: i32, female: bool) {
        let act_index = if female { 10 } else { 0 } + kind as usize;
        match set {
            0 => { if self.inf0a[act_index] == 0 { self.inf0a[act_index] = index; } }
            1 => { if self.cav0b[act_index] == 0 { self.cav0b[act_index] = index; } }
            2 => { if self.cav2c[act_index] == 0 { self.cav2c[act_index] = index; } }
            3 => { if self.wng2d[act_index] == 0 { self.wng2d[act_index] = index; } }
            4 => { if self.wng0e[act_index] == 0 { self.wng0e[act_index] = index; } }
            5 => { if self.wng1f[act_index] == 0 { self.wng1f[act_index] = index; } }
            _ => {}
        }
    }
    pub fn get(&self, mount: Mount, female: bool, kind: i32) -> Option<&'static Il2CppString>{
        let act_index = if female { 10 } else { 0 } + kind as usize;
        let anim_index = 
        match Mount {
            Mount::None => { self.inf0a[act_index] }
            Mount::Cav => { self.cav0b[act_index] }
            Mount::Wolf => { self.cav2c[act_index] }
            Mount::Wyvern => { self.wng2d[act_index] }
            Mount::Pegasus => { self.wng0e[act_index] }
            Mount::Griffin => { self.wng1f[act_index] }
        };

        if anim_index > 0 { AnimSetDB::try_index_get(anim_index).map_or_else(|| None, |f| Some(f.name)) }
        else { None }
    }
}
*/
impl AnimData {
    pub fn create_anim(&self, item_kind: i32, is_morph: bool) -> String {
        let mut suffix = if self.is_generic { 
            if item_kind == 4 && ( self.weapon_mask & 14 == 0)  { "c000_L" }
            else if item_kind == 6 && ( self.weapon_mask & 14 == 0) { "c000_M"}
            else { "c000_N" }
        }
        else { self.suffix.as_str() }.to_string();
        if suffix.contains("_M") && self.weapon_mask & 14 != 0 { suffix = suffix.replace("_M", "_N"); }
        let gen = if self.gender == Gender::Male { "M" } else { "F" }; 

        if item_kind == 9 { 
            if self.special != 0 { return concat_string!(self.act_name, "-", WEP_PRE[self.special as usize], "2_", suffix); }
            if self.has_transformation { return concat_string!(self.act_name, "-No2_", suffix); }
        }
        if self.weapon_mask & (1 << item_kind ) != 0 { concat_string!(self.act_name, "-", WEP_PRE[self.special as usize], "1_", suffix); 
            let out = concat_string!(self.act_name, "-", WEP_PRE[item_kind as usize], "1_", suffix);
            println!("Constructed BodyAnim: {}", out);
            return out;
        }
        else {
            match self.mount {
                Mount::Cav => { 
                    match item_kind {
                        1|2|3 => { return concat_string!("Cav0B", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N"); }
                        4 => { return concat_string!("Bow2B", gen, "-", WEP_PRE[item_kind as usize], "1_c000_L"); }
                        6 => { return concat_string!("Mag2BM-Mg1_c000_M"); }
                        7 => { return concat_string!("Com0B", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N"); }
                        5|8|9 => { return concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_N"); }
                        _ => {  return concat_string!("Com0B", gen, "-No1_c000_N"); }
                    }
                }
                Mount::Pegasus => { 
                    // Com0XF-F
                    // UAS_XXX#DG
                    match item_kind {
                        1|2|3|7 => { return concat_string!("Wng0EF-", WEP_PRE[item_kind as usize], "1_c000_N"); }
                        6 => { return String::from("Slp0EF-Mg1_c351_M"); }
                        4|5|8|9 => { return concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_", if item_kind ==  4 { "L" } else { "N"}); }
                        _ => { return String::from("Wng0EF-No1_c000_N"); }
                    }
                }
                Mount::Wolf => { 
                    match item_kind {
                        1|2|3|5|7 => { return concat_string!("Cav2C", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N"); }
                        4|6|8|9 =>  { return concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_", if item_kind ==  4 { "L" } else { "N"}); }
                        _ => {  return concat_string!("Com0B", gen, "-No1_c000_N"); }
                    }
                }
                Mount::Wyvern => { 
                    match item_kind {
                        1|2|3|7 => { return concat_string!("Wng2D", gen, "-", WEP_PRE[item_kind as usize], "1_c000_N"); }
                        6 => {
                            if gen == "F" { return String::from("Cmi0DF-Mg1_c561_M") }
                            else { return String::from("Mag1AM-Mg1_c000_M") }
                        }
                        4|5|8|9 => { return concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_", if item_kind ==  4 { "L" } else { "N"}); }
                        _ => { return concat_string!("Wng2D", gen, "-No1_c000_N"); }
                    }
                }
                _ => { return concat_string!(INF_ACT[item_kind as usize], gen, "-", WEP_PRE[item_kind as usize], "1_c000_N"); }
            }
        }
    }
}
