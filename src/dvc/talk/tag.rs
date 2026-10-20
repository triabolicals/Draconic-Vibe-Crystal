use engage::app::IJobDataMethods;
pub use super::*;
use engage::{
    app::{
        mapmind::*, persondata::*,
        ISingletonClass_1Methods, GodPool, IGodUnitMethods, ISingletonProcInst_1Methods,
        IStructBase, IStructData_1Methods, IUnitMethods,
        Mess, UnitPool,
    },
    GameVariableManager,
};
use crate::{
    DVCFlags, DVCVariables, dvc::asset::AppearanceManager,
    enums::PIDS, gamedata::{EmblemPool, PersonPool}, utils::to_opt,
    dvc::mess::{MessageList, MESS_REPLACEMENT_DATA}, DVCFlags::GodNames, enums::{EMBLEM_GIDS, RINGS}
};

#[unity::inject(namespace="App.Talk3D", name="TalkTagTextReplace", parent=TalkTagName)]
pub struct TalkTagTextReplace {
    pub replacement: i32,
}
impl TalkTagTextReplace {
    pub fn get_replacement_type(self) -> TextReplacementKind { TextReplacementKind::from_i32(self.replacement()) }
    pub fn set_replacement_type(self, replacement_type: TextReplacementKind) { self.set_replacement(replacement_type.i32()); }
    pub fn set_replacement_from_tag(self, tag: u16) { self.set_replacement_type(TextReplacementKind::from_mess_tag(tag)); }
}
#[unity::injected_methods]
impl TalkTagTextReplace {
    #[override_virtual(name="Execute")]
    pub fn execute(self) {
        if self.get_replacement_type() != TextReplacementKind::Pass {
            let talk = TalkSequence::get_instance();
            talk.set_replace_text(self.m_replace_name());
        }
    }
    #[override_virtual(name="Initialize")]
    pub fn initialize(self, talk_ptr: TalkPtr){
        let tag_id = talk_ptr.read_int16();
        let length = talk_ptr.read_int16(); // Command Length
        let i = tag_id & 255;
        let i2 = (tag_id >> 8) & 255;
        let line_break = if length > 0 { Some(talk_ptr.read_int16() as u16) } else { None };
        let text = self.get_replacement_type().get_text(i, i2, line_break);
        self.set_m_replace_name(text.into());
    }
}
#[repr(i32)]
#[derive(PartialEq, Clone, Copy, PartialOrd)]
pub enum TextReplacementKind {
    UnitName,
    EmblemName,
    EmblemRing,
    EmblemAlias,
    EmblemInvocation,
    HeroAlias,
    UnitClass,
    Common,
    TargetName,
    EmblemRingTile,
    EmblemNameNoSwap,
    UnitNameNoSwap,
    GenderStringNoSwap,
    GenderStringSwap,
    DemoPID,
    DemoGID,
    DemoCustomEmblem,
    DemoPIDNoSwap,
    DemoGIDNoSwap,
    HeroJob,
    RingParalogueSwap,
    Pass,
}
impl TextReplacementKind {
    pub fn from_mess_tag(tag: u16) -> Self {
        match tag {
            15 => TextReplacementKind::UnitName,
            16 => TextReplacementKind::EmblemName,
            17 => TextReplacementKind::EmblemRing,
            18 => TextReplacementKind::EmblemAlias,
            19 => TextReplacementKind::EmblemInvocation,
            20 => TextReplacementKind::HeroAlias,
            21 => TextReplacementKind::UnitClass,
            22 => TextReplacementKind::Common,
            23 => Self::EmblemRingTile,
            24 => Self::TargetName,
            25 => Self::EmblemNameNoSwap,
            26 => Self::UnitNameNoSwap,
            27 => Self::GenderStringNoSwap,
            28 => Self::GenderStringSwap,
            29 => Self::DemoPID,
            30 => Self::DemoGID,
            31 => Self::DemoCustomEmblem,
            32 => Self::DemoPIDNoSwap,
            33 => Self::DemoGIDNoSwap,
            34 => Self::HeroJob,
            35 => Self::RingParalogueSwap,
            _ => TextReplacementKind::Pass,
        }
    }
    pub fn to_mess_tag(self) -> u16 {
        match self {
            TextReplacementKind::UnitName => 15,
            TextReplacementKind::EmblemName => 16,
            TextReplacementKind::EmblemRing => 17,
            TextReplacementKind::EmblemAlias => 18,
            TextReplacementKind::EmblemInvocation => 19,
            TextReplacementKind::HeroAlias => 20,
            TextReplacementKind::UnitClass => 21,
            TextReplacementKind::Common => 22,
            TextReplacementKind::EmblemRingTile => 23,
            TextReplacementKind::TargetName => 24,
            Self::EmblemNameNoSwap => 25,
            Self::UnitNameNoSwap => 26,
            Self::GenderStringNoSwap => 27,
            Self::GenderStringSwap => 28,
            TextReplacementKind::DemoPID => 29,
            TextReplacementKind::DemoGID => 30,
            TextReplacementKind::DemoCustomEmblem => 31,
            Self::DemoPIDNoSwap => 32,
            Self::DemoGIDNoSwap => 33,
            Self::HeroJob => 34,
            Self::RingParalogueSwap => 35,
            _ => 49,
        }
    }
    pub fn i32(self) -> i32 { self as i32 }
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => TextReplacementKind::UnitName,
            1 => TextReplacementKind::EmblemName,
            2 => TextReplacementKind::EmblemRing,
            3 => TextReplacementKind::EmblemAlias,
            4 => TextReplacementKind::EmblemInvocation,
            5 => TextReplacementKind::HeroAlias,
            6 => TextReplacementKind::UnitClass,
            7 => TextReplacementKind::Common,
            8 => Self::TargetName,
            9 => Self::EmblemRingTile,
            10 => Self::EmblemNameNoSwap,
            11 => Self::UnitNameNoSwap,
            12 => Self::GenderStringNoSwap,
            13 => Self::GenderStringSwap,
            14 => TextReplacementKind::DemoPID,
            15 => TextReplacementKind::DemoGID,
            16 => TextReplacementKind::DemoCustomEmblem,
            17 => Self::DemoPIDNoSwap,
            18 => Self::DemoGIDNoSwap,
            19 => Self::HeroJob,
            20 => Self::RingParalogueSwap,
            _ => Self::Pass,
        }
    }
    pub fn create_tag(self, tag_index: impl Into<u16>) -> [u16; 4] { [14, self.to_mess_tag(), tag_index.into(), 0] }
    pub fn create_tag_with_flag(self, tag_index: impl Into<u16>, break_pos: u16, all_upper: bool) -> [u16; 5] {
        let v = if all_upper { (break_pos & 127) | 128 } else { break_pos & 127 };
        [14, self.to_mess_tag(), tag_index.into(), 2, v]
    }
    pub fn get_text(self, arg1: i32, arg2: i32, line_break_position: Option<u16>) -> String {
        let mut name = String::new();
        let all_upper = line_break_position.map(|v| v & 128 != 0).unwrap_or(false);
        let line_break_position = line_break_position.map(|v| v & 127).filter(|p| *p > 0);
        match self {
            TextReplacementKind::HeroJob => {
                if let Some(unit) = to_opt(UnitPool::get_hero(false)){
                    let mpid = unit.get_person().get_name().to_rust_string().replace("MPID_", "MPID_alias_");
                    if Mess::is_exist(mpid.as_str()) { name = Mess::get(mpid.as_str()).to_rust_string(); }
                    else { name = Mess::get(unit.get_job().get_name()).to_rust_string(); }
                }
                else { name = Mess::get("MPID_alias_Lueur").to_string(); }
            }
            TextReplacementKind::UnitName => {
                name = MESS_REPLACEMENT_DATA.get().unwrap().read().unwrap().person_list[arg1 as usize].string.clone();
                match arg1 {
                    0..41|42..45 => {
                        let index = if arg1 > 41 { arg1 - 4 } else { arg1 };
                        if let Some(replacement) = PersonPool::get_replacement(index, false, true){
                            if replacement.hash() == PersonPool::LUEUR { name = GameVariableManager::get_string(DVCVariables::LUEUR_NAME).to_rust_string(); }
                            else if let Some(unit) = to_opt(UnitPool::get_from_person(replacement, false)) { name = unit.get_name().to_rust_string(); }
                            else { name = Mess::get(replacement.get_name()).to_rust_string(); }
                        }
                    }
                    41 => { name = Mess::get("MPID_Il").to_rust_string(); }
                    _ => {
                        if DVCFlags::RandomBossesNPCs.get_value(){
                            let npc_index = arg1 + 100;
                            if let Some(appearance) = PersonPool::get().hash_to_index.iter().find(|(_, index)| **index == npc_index)
                                .and_then(|(hash, _)| to_opt(PersonData::try_get_from_hash(*hash)))
                                .and_then(|person| AppearanceManager::get_appearance_name(person, None))
                            {
                                name = appearance.to_rust_string();
                            }
                        }
                    }
                }
            }
            TextReplacementKind::EmblemRingTile => {
                if let Some(replacement) = EmblemPool::get_replacement_god_3(arg1, false) {
                    name = Mess::get(replacement.get_ringname()).to_rust_string();
                }
                else { name = Mess::get(format!("MGID_Ring_{}", RINGS[arg1 as usize])).to_rust_string(); }
            }
            TextReplacementKind::EmblemName => {
                name = Mess::get(format!("MGID_{}", RINGS[arg1 as usize])).to_rust_string();
                if arg1 == 20 {
                    if let Some(v) = EmblemPool::get_replacement_index_2(12, false).filter(|&v| v == 11 || v == 18){
                        if v == 11 { name = Mess::get("MGID_Ephraim").to_rust_string(); }
                        else { name = Mess::get("MGID_Robin").to_rust_string(); }
                    }
                    else { name = Mess::get("MGID_Dimitri").to_rust_string(); }
                }
                else if let Some(god) = EmblemPool::get_replacement_god_3(arg1, false) {
                    if let Some(god_unit) = to_opt(GodPool::try_get_2(god, false)) { name = god_unit.get_name().to_rust_string(); }
                    else { name = Mess::get(god.get_mid()).to_rust_string(); }
                }
            }
            TextReplacementKind::EmblemAlias => {
                let emblem = arg1 % 20;
                let kind = arg1 / 20;
                let data =  MESS_REPLACEMENT_DATA.get().unwrap().read().unwrap();
                if let Some(replace) = EmblemPool::get_replacement_index_2(emblem, false).filter(|r| *r < 19){
                    name = data.emblem_alias[(20*kind + replace) as usize].string.clone();
                }
                else { name = data.emblem_alias[arg1 as usize].string.clone(); }
            }
            TextReplacementKind::EmblemRing => {
                let data =  MESS_REPLACEMENT_DATA.get().unwrap().read().unwrap();
                name = EmblemPool::get_replacement_index_2(arg1, false)
                    .and_then(|replace| {
                        let index = if replace >= 19 { replace - 1 } else { replace };
                        data.rings.get(index as usize).map(|v| v.string.clone() )
                    })
                    .unwrap_or_else(|| data.rings[arg1 as usize].string.clone() );
            }
            TextReplacementKind::HeroAlias => {
                let data = MessageList::get();
                if let Some(index) = PersonPool::get_replacement_index(0, false) {
                    if index < 41 { name = data.alias[index as usize].string.clone(); }
                    else { name = TextReplacementKind::UnitName.get_text(0, arg2, None); }
                }
                else { name = data.alias[0].string.clone(); }
            }
            TextReplacementKind::UnitClass => {}
            TextReplacementKind::Common => {}
            TextReplacementKind::DemoPID => {
                if let Some(pid) = PersonPool::get_replacement(arg1, false, false).map(|v| v.get_pid().to_rust_string()) { name = pid; }
                else { name = PIDS[arg1 as usize].to_string(); }
            }
            TextReplacementKind::DemoGID => {
                if let Some(gid) = EmblemPool::get_replacement_god_3(arg1, false).map(|v| v.get_gid().to_rust_string()) { name = gid; }
                else { name = EMBLEM_GIDS[arg1 as usize].to_string(); }
            }
            TextReplacementKind::DemoGIDNoSwap => {
                name = EMBLEM_GIDS[arg1 as usize].to_string();
            }
            TextReplacementKind::DemoCustomEmblem => {
                if let Some(gid) = EmblemPool::get_replacement_god_3(arg1, false).filter(|p| EmblemPool::is_custom(*p)).map(|p| p.get_gid().to_rust_string()) { name = gid; }
                else { name = EMBLEM_GIDS[arg1 as usize].to_string(); }
            }
            TextReplacementKind::TargetName => {
                if let Some(target) = to_opt(MapMind::get_instance()).and_then(|m| to_opt(m.get_target_unit())){
                    name = target.get_name().to_rust_string();
                }
                else { name = Mess::get_argument(0).to_rust_string(); }
            }
            TextReplacementKind::EmblemNameNoSwap => {
                if let Some(god) = EmblemPool::get().emblem_data.get(arg1 as usize).and_then(|m| to_opt(m.get_data())) {
                    name = if GodNames.get_value() { AppearanceManager::get_emblem_appearance_name(arg1) } else { Mess::get(god.get_mid()) }.to_rust_string();
                }
                else { name = MESS_REPLACEMENT_DATA.get().unwrap().read().unwrap().emblem_list[arg1 as usize].string.clone(); }
            }
            Self::DemoPIDNoSwap => { name = PersonPool::get().pids[arg1 as usize].0.clone(); }
            Self::UnitNameNoSwap => {
                if arg1 == 0 {
                    if let Some(lueur) = to_opt(UnitPool::get_from_person_2(PIDS[0], false)).map(|v| v.get_name()) { name = lueur.to_rust_string(); }
                    else { name = GameVariableManager::get_string(DVCVariables::LUEUR_NAME).to_rust_string(); }
                }
                else { name = Mess::get(PersonPool::get().playable[arg1 as usize].get_person_data().get_name()).to_rust_string(); }
            }
            Self::GenderStringSwap|Self::GenderStringNoSwap => {
                if arg1 < PersonPool::get().playable.len() as i32 {
                    let arg1 = if self == Self::GenderStringSwap { PersonPool::get_replacement_index(arg1, false).unwrap_or(arg1) } else { arg1 };
                    let female = if arg1 == 0 { DVCVariables::is_lueur_female() } else { PersonPool::get().playable[arg1 as usize].female } as i32;
                    let honor = arg2 < 3;
                    name = MESS_REPLACEMENT_DATA.get().unwrap().read().unwrap().gender[arg2 as usize].get(female+1, honor).to_string();
                }
                else { return "???".to_string(); }
            }
            Self::RingParalogueSwap => {
                if let Some(god) = EmblemPool::get_replacement_god_3(arg1, false).filter(|v| EmblemPool::is_custom(*v)) { name = Mess::get(god.get_mid()).to_rust_string(); }
                else { name = Mess::get(format!("MGID_{}", EmblemPool::MGIDS[arg1 as usize])).to_rust_string(); }
            }
            _ => {}
        }
        // #[cfg(feature = "debug")] println!("TagReplace {}({}) -> {}", self.to_mess_tag(), arg1, name);
        if all_upper { name = name.to_uppercase(); }
        if let Some(line_break) = line_break_position {
            let mut new = String::new();
            let mut added = false;
            let break_pos = (line_break & 31) as usize;
            name.split_whitespace().for_each(|s|{
                if !new.is_empty() {
                    if new.len() >= break_pos && !added {
                        new += "\n";
                        added = true;
                    }
                    else { new += " "; }
                }
                new += s;
            });
            if new.is_empty() { new = name; }
            if line_break & 32 != 0 { new += if added { " " } else { "\n" }; }
            else if line_break & 64 != 0 { new += " "; }
            new
        }
        else { name }
    }
}

