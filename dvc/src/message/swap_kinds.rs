use engage::gamedata::{item::ItemData, Gamedata};

#[derive(Clone, PartialEq)]
pub enum MessSwapType {
    HeroAlias(bool),
    HeroJob,
    UnitJob(u16, u16),
    UnitName(u16),
    EmblemName(u16),
    RingName(u16),
    EmblemAlias(u16),
    EmblemInvocation(u16),
    UnitGenderTextSwap{ person_idx: u16, txt_idx: u16 },
    EmblemGenderTextSwap{ emblem_idx: u16, txt_idx: u16 },
    ItemKind(u16),
    LiberationKind,
    RingBracelet(u16),
    UnitAlias(u16),
    Skip,
}

impl MessSwapType {
    pub fn from_iter(iter: &mut std::str::SplitWhitespace) -> Option<Self> {
        let a = iter.next()?;
        let lower = a.to_lowercase();
        match lower.as_str() {
            "s" => Some(MessSwapType::Skip),
            "a"|"alias" => {
                // println!("Find Alias");
                Some(MessSwapType::HeroAlias(false))
            }
            "j"|"j2" => { Some(MessSwapType::HeroJob) }
            "u"|"unit" => {
                let index = iter.next().and_then(|i| i.parse::<u16>().ok())?;
                // println!("Found Unit Swap with: {}", index);
                Some(MessSwapType::UnitName(index))
            }
            "e"|"emblem" => {
                let index = iter.next().and_then(|i| i.parse::<u16>().ok()).filter(|i| *i < 24)?;
                Some(MessSwapType::EmblemName(index))
            }
            "r"|"ring"|"e0" => {
                let index = iter.next().and_then(|i| i.parse::<u16>().ok()).filter(|i| *i < 20)?;
                Some(MessSwapType::RingName(index))
            }
            "ea"|"e1" => {
                let index = iter.next().and_then(|i| i.parse::<u16>().ok()).filter(|i| *i < 20)?;
                Some(MessSwapType::EmblemAlias(index))
            }
            "ei"|"e2" => {
                let index = iter.next().and_then(|i| i.parse::<u16>().ok()).filter(|i| *i < 20)?;
                Some(MessSwapType::EmblemInvocation(index))
            }
            "gu"|"gender-unit" => {
                let person_idx = iter.next().and_then(|i| i.parse::<u16>().ok())?;
                let txt_idx = iter.next().and_then(|i| i.parse::<u16>().ok())?;
                Some(MessSwapType::UnitGenderTextSwap { person_idx, txt_idx })
            }
            "ge"|"gender-emblem" => {
                let emblem_idx = iter.next().and_then(|i| i.parse::<u16>().ok())?;
                let txt_idx = iter.next().and_then(|i| i.parse::<u16>().ok())?;
                Some(MessSwapType::EmblemGenderTextSwap { emblem_idx, txt_idx })
            }
            "ik" => { 
                let item_index = iter.next().and_then(|iid| ItemData::get(iid)).map(|v| v.parent.index)?;
                // println!("Found Item Kind Swap with: {}", Mess::get_name(ItemData::try_index_get(item_index).unwrap().iid));
                Some(MessSwapType::ItemKind(item_index as u16))
            }
            "lib" => {
                Some(MessSwapType::LiberationKind)
            }
            "br" => {
                let emblem_idx = iter.next().and_then(|i| i.parse::<u16>().ok())?;
                Some(MessSwapType::RingBracelet(emblem_idx))
            }
            "ju" => {
                let person_index = iter.next().and_then(|i| i.parse::<u16>().ok().filter(|i| *i < 41))?;
                let text_idx = iter.next().and_then(|i| i.parse::<u16>().ok())?;
                Some(MessSwapType::UnitJob(person_index, text_idx))
            }
            _ => None,
        }
    }
    pub fn create_tag_arguments(&self, is_upper: bool, new_line_pos: usize) -> Vec<u16> {
        let mut out = vec![
            14, 6, self.get_id(),
            2 * (1 + self.get_arg_count()) as u16
        ];
        match self {
            MessSwapType::UnitName(idx) => { return vec![14, 6, 100+*idx, 0]; }
            MessSwapType::EmblemName(idx) => { return vec![14, 6, 200+*idx, 0]; }
            MessSwapType::RingName(idx) => { return vec![14, 6, 300+*idx, 0]; }
            MessSwapType::EmblemAlias(idx) => { return vec![14, 6, 320+*idx, 0]; }
            MessSwapType::EmblemInvocation(idx) => { return vec![14, 6, 340+*idx, 0]; }
            MessSwapType::UnitJob(person, _) => { return vec![14, 6, self.get_id(), 2, *person]; }
            MessSwapType::ItemKind(kind) => out.extend([*kind, 0]),
            MessSwapType::LiberationKind => out.push(1),
            MessSwapType::HeroAlias(alt) => out.push( *alt as u16),
            MessSwapType::HeroJob  => { },
            MessSwapType::UnitGenderTextSwap { person_idx, txt_idx } =>  out.extend([*person_idx, *txt_idx, is_upper as u16]),
            MessSwapType::EmblemGenderTextSwap { emblem_idx, txt_idx } =>  out.extend([*emblem_idx, *txt_idx, is_upper as u16]),
            MessSwapType::RingBracelet(emblem_idx) => out.extend([*emblem_idx, is_upper as u16]),
            _ => {}
        }
        out.push(new_line_pos as u16);
        // println!("Tag: {} {} {} {}", out[0], out[1], out[2], out[3]);
        out
    }
    pub fn get_id(&self) -> u16 {
        match self {
            MessSwapType::UnitName(_) => 10,
            MessSwapType::EmblemName(_) => 11,
            MessSwapType::RingName(_) => 12,
            MessSwapType::EmblemAlias(_) => 13,
            MessSwapType::EmblemInvocation(_)=> 14,
            MessSwapType::HeroAlias(_) => 16,
            MessSwapType::UnitGenderTextSwap { person_idx: _, txt_idx: _ } => 17,
            MessSwapType::EmblemGenderTextSwap { emblem_idx: _, txt_idx: _ } => 18,
            MessSwapType::HeroJob => 19,
            MessSwapType::LiberationKind => 20,
            MessSwapType::ItemKind(_) => 21,
            MessSwapType::RingBracelet(_) => 22,
            MessSwapType::UnitAlias(_) => 23,
            MessSwapType::UnitJob(_, _) => 24,
            MessSwapType::Skip => 0,
        }
    }
    pub fn get_arg_count(&self) -> usize {
        match self {
            MessSwapType::HeroAlias(_) => 1, 
            MessSwapType::HeroJob => 0,
            MessSwapType::LiberationKind => 1,
            MessSwapType::UnitName(_) => 1,
            MessSwapType::UnitJob(_, _) => 2, 
            MessSwapType::RingBracelet(_) => 2,
            MessSwapType::EmblemName(_)|MessSwapType::EmblemAlias(_) => 1,
            MessSwapType::RingName(_)|MessSwapType::EmblemInvocation(_) => 1,
            MessSwapType::UnitGenderTextSwap { person_idx: _, txt_idx: _ } => 3,
            MessSwapType::EmblemGenderTextSwap { emblem_idx: _x, txt_idx: _ } => 3,
            MessSwapType::ItemKind(_) => 2,
            _ => 0,
        }
    }
    pub fn check_capitalization(&self) -> bool {
        match self {
            MessSwapType::UnitGenderTextSwap{ person_idx: _, txt_idx: _ } => { true }
            MessSwapType::EmblemGenderTextSwap{ emblem_idx: _, txt_idx: _ } => { true }
            _ => false
        }
    }

}


