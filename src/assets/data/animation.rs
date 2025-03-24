use super::*;

pub const ACT_PRE: &[&str] = &[
    "Com0AM", "Com0AF",
    "Com0BM", "Com0BF",
    "Swd0AM", "Swd1AM", "Swd2AM",
    "Lnc0AM", "Lnc1AM", "Lnc2BM",
    "Axe0AM", "Axe1AM", "Axe2AM",
    "Bow0AM", "Bow1AM", "Bow2BM",
    "Dge0AM",
    "Mag0AM", "Mag1AM", "Mag2BM",
    "Rod0AM", "Rod1AM", "Rod2AM",
    "Cav0BM", "Cav1BM", "Cav2CM",
    "Amr0AM", "Amr1AM", "Amr2B",
    "Wng1FM", "Wng2DM", 

    "Swd0AF", "Swd1AF", "Swd2AF",
    "Lnc0AF", "Lnc1AF", "Lnc2BF",
    "Axe0AF", "Axe1AF", "Axe2AF",
    "Bow0AF", "Bow1AF", "Bow2BF",
    "Dge0AF",
    "Mag0AF", "Mag1AF", "Mag2BF",
    "Rod0AF", "Rod1AF", "Rod2AF",
    "Cav0BF", "Cav1BF", "Cav2CF",
    "Amr0AF", "Amr1AF", "Amr2BF",
    "Wng1FM", "Wng2DM",
    "Drg0AM", "Drg0AF", "Drg1AM", "Drg1AF", 
    "Sdp0AF",
    "Sdk0AM",
    "Avn0BM", "Avn1BM",
    "Flr0AF", "Flr1AF",
    "Scs0AM", "Scs1AM",
    "Trl0AM", "Trl1AM",
    "Lnd0DF", "Lnd1DF",
    "Slp0EF", "Slp1EF",
    "Pcf0AF", "Pcf1AF",
    "Cpd0BM", "Cpd1BM", 
    "Dnc0AM",
    "Sds0AM", "Sds0AF",
    "Msn0DF", "Msn1DF",
    "Ect3AM", "Ect3AF",
    "Mcn3AM", "Mcn3AF",
    // Backup/Dragon/Cav, Qi, Flying, Magic, Armor
    "Enb0AM", "Enc0AM", "Enw0AM", "Enm0AM", "Enh0AM", 
    "Enb0AF", "Enc0AF", "Enw0AF", "Enm0AF", "Enh0AF", 
    "Mar0AM", "Sig0BM", "Cel0AF", "Mic0AF", "Roy0AM", "Lei0AM", "Luc0AF", "Lyn0AF", "Ike0AM", "Byl0AM", "Cor0AM", "Eir0AF", "Eph0AF", "Ler1AM", "Ler1AF",
    "Tik0AF", "Ede0AF", "Dim0AM", "Cla0AM", "Hec0AM", "Ver0AF", "Sor0AM", "Cmi0DF", "Chr0AM", "Rbi0AM"
];



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

