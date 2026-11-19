use core::fmt;

use super::MAX_COMPANIONS;
use crate::code::{
    ApiFramework, BannerType, CompanionType, CreativeAttribute, DeliveryMethod,
    ExpandableDirection, FeedType, Linearity, Placement, PlacementPosition, Protocol, VcmMode,
    VolumeNormalization,
};
use crate::flag::Flag;
use crate::ids::BannerId;
use crate::raw::RawJson;
use crate::write::{
    Object, object, option_id, option_number, option_number_array, option_raw, option_string_array,
};

/// A banner impression or a video companion ad.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Banner {
    w: Option<u32>,
    h: Option<u32>,
    btype: &'static [BannerType],
    battr: &'static [CreativeAttribute],
    pos: Option<PlacementPosition>,
    mimes: &'static [&'static str],
    topframe: Option<Flag>,
    expdir: &'static [ExpandableDirection],
    api: &'static [ApiFramework],
    id: Option<BannerId>,
    vcm: Option<VcmMode>,
    ext: Option<RawJson>,
}

setters!(Banner {
    set_w => w: Option<u32>,
    set_h => h: Option<u32>,
    set_btype => btype: &'static [BannerType],
    set_battr => battr: &'static [CreativeAttribute],
    set_pos => pos: Option<PlacementPosition>,
    set_mimes => mimes: &'static [&'static str],
    set_topframe => topframe: Option<Flag>,
    set_expdir => expdir: &'static [ExpandableDirection],
    set_api => api: &'static [ApiFramework],
    set_id => id: Option<BannerId>,
});

impl Banner {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_number(f, "w", self.w)?;
        option_number(f, "h", self.h)?;
        option_number_array(f, "btype", self.btype)?;
        option_number_array(f, "battr", self.battr)?;
        option_number(f, "pos", self.pos)?;
        option_string_array(f, "mimes", self.mimes)?;
        option_number(f, "topframe", self.topframe)?;
        option_number_array(f, "expdir", self.expdir)?;
        option_number_array(f, "api", self.api)?;
        option_id(f, "id", self.id)?;
        option_number(f, "vcm", self.vcm)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            w: Some(300),
            h: Some(250),
            btype: &[BannerType::IFRAME],
            battr: &[CreativeAttribute::AUDIO_AUTOPLAY],
            pos: Some(PlacementPosition::ABOVE_THE_FOLD),
            mimes: &["image/jpeg"],
            topframe: Some(Flag::Yes),
            expdir: &[ExpandableDirection::UP],
            api: &[ApiFramework::MRAID_2],
            id: Some(BannerId::new(7)),
            vcm: Some(VcmMode::END_CARD),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}
/// A fixed-capacity list of companion banners.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BannerList {
    items: [Banner; MAX_COMPANIONS],
    len: u8,
}

impl Default for BannerList {
    fn default() -> Self {
        Self { items: [Banner::default(); MAX_COMPANIONS], len: 0 }
    }
}

impl BannerList {
    pub(crate) fn push(&mut self, banner: &Banner) {
        if let Some(slot) = self.items.get_mut(usize::from(self.len)) {
            *slot = *banner;
            self.len = self.len.saturating_add(1);
        }
    }

    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>, name: &str) -> fmt::Result {
        if self.len == 0 {
            return Ok(());
        }

        f.key(name)?;
        f.out().write_char('[')?;

        let mut first = true;
        for banner in self.items.iter().take(usize::from(self.len)) {
            if first {
                first = false;
            } else {
                f.out().write_char(',')?;
            }
            object(f.out(), |f| banner.write(f))?;
        }

        f.out().write_char(']')
    }
}

/// A video impression.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Video {
    mimes: &'static [&'static str],
    minduration: Option<u32>,
    maxduration: Option<u32>,
    protocols: &'static [Protocol],
    w: Option<u32>,
    h: Option<u32>,
    startdelay: Option<i32>,
    placement: Option<Placement>,
    linearity: Option<Linearity>,
    skip: Option<Flag>,
    skipmin: Option<u32>,
    skipafter: Option<u32>,
    api: &'static [ApiFramework],
    battr: &'static [CreativeAttribute],
    delivery: &'static [DeliveryMethod],
    pos: Option<PlacementPosition>,
    companionad: BannerList,
    companiontype: &'static [CompanionType],
    ext: Option<RawJson>,
}

setters!(Video {
    set_mimes => mimes: &'static [&'static str],
    set_minduration => minduration: Option<u32>,
    set_maxduration => maxduration: Option<u32>,
    set_protocols => protocols: &'static [Protocol],
    set_w => w: Option<u32>,
    set_h => h: Option<u32>,
    set_startdelay => startdelay: Option<i32>,
    set_placement => placement: Option<Placement>,
    set_linearity => linearity: Option<Linearity>,
    set_skip => skip: Option<Flag>,
    set_skipmin => skipmin: Option<u32>,
    set_skipafter => skipafter: Option<u32>,
    set_api => api: &'static [ApiFramework],
    set_battr => battr: &'static [CreativeAttribute],
    set_delivery => delivery: &'static [DeliveryMethod],
    set_pos => pos: Option<PlacementPosition>,
    set_companionad => companionad: BannerList,
    set_companiontype => companiontype: &'static [CompanionType],
});

impl Video {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_string_array(f, "mimes", self.mimes)?;
        option_number(f, "minduration", self.minduration)?;
        option_number(f, "maxduration", self.maxduration)?;
        option_number_array(f, "protocols", self.protocols)?;
        option_number(f, "w", self.w)?;
        option_number(f, "h", self.h)?;
        option_number(f, "startdelay", self.startdelay)?;
        option_number(f, "placement", self.placement)?;
        option_number(f, "linearity", self.linearity)?;
        option_number(f, "skip", self.skip)?;
        option_number(f, "skipmin", self.skipmin)?;
        option_number(f, "skipafter", self.skipafter)?;
        option_number_array(f, "api", self.api)?;
        option_number_array(f, "battr", self.battr)?;
        option_number_array(f, "delivery", self.delivery)?;
        option_number(f, "pos", self.pos)?;
        self.companionad.write(f, "companionad")?;
        option_number_array(f, "companiontype", self.companiontype)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) fn maximal() -> Self {
        let mut companionad = BannerList::default();
        companionad.push(&Banner::maximal());

        Self {
            mimes: &["video/mp4"],
            minduration: Some(5),
            maxduration: Some(30),
            protocols: &[Protocol::VAST_2_0, Protocol::VAST_3_0],
            w: Some(640),
            h: Some(480),
            startdelay: Some(-1),
            placement: Some(Placement::IN_STREAM),
            linearity: Some(Linearity::LINEAR),
            skip: Some(Flag::Yes),
            skipmin: Some(5),
            skipafter: Some(5),
            api: &[ApiFramework::MRAID_2],
            battr: &[CreativeAttribute::AUDIO_AUTOPLAY],
            delivery: &[DeliveryMethod::PROGRESSIVE],
            pos: Some(PlacementPosition::ABOVE_THE_FOLD),
            companionad,
            companiontype: &[CompanionType::STATIC],
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// An audio impression.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Audio {
    mimes: &'static [&'static str],
    minduration: Option<u32>,
    maxduration: Option<u32>,
    protocols: &'static [Protocol],
    startdelay: Option<i32>,
    feed: Option<FeedType>,
    stitched: Option<Flag>,
    nvol: Option<VolumeNormalization>,
    api: &'static [ApiFramework],
    battr: &'static [CreativeAttribute],
    delivery: &'static [DeliveryMethod],
    ext: Option<RawJson>,
}

impl Audio {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_string_array(f, "mimes", self.mimes)?;
        option_number(f, "minduration", self.minduration)?;
        option_number(f, "maxduration", self.maxduration)?;
        option_number_array(f, "protocols", self.protocols)?;
        option_number(f, "startdelay", self.startdelay)?;
        option_number(f, "feed", self.feed)?;
        option_number(f, "stitched", self.stitched)?;
        option_number(f, "nvol", self.nvol)?;
        option_number_array(f, "api", self.api)?;
        option_number_array(f, "battr", self.battr)?;
        option_number_array(f, "delivery", self.delivery)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            mimes: &["audio/mp4"],
            minduration: Some(5),
            maxduration: Some(30),
            protocols: &[Protocol::VAST_2_0, Protocol::VAST_3_0],
            startdelay: Some(-1),
            feed: Some(FeedType::MUSIC_STREAM),
            stitched: Some(Flag::Yes),
            nvol: Some(VolumeNormalization::AVERAGE),
            api: &[ApiFramework::MRAID_2],
            battr: &[CreativeAttribute::AUDIO_AUTOPLAY],
            delivery: &[DeliveryMethod::PROGRESSIVE],
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// A native impression.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Native {
    request: &'static str,
    ver: &'static str,
    api: &'static [ApiFramework],
    battr: &'static [CreativeAttribute],
    ext: Option<RawJson>,
}

setters!(Native {
    set_request => request: &'static str,
    set_ver => ver: &'static str,
    set_api => api: &'static [ApiFramework],
    set_battr => battr: &'static [CreativeAttribute],
    set_ext => ext: Option<RawJson>,
});

impl Native {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        crate::write::field_string(f, "request", self.request)?;
        crate::write::field_string(f, "ver", self.ver)?;
        option_number_array(f, "api", self.api)?;
        option_number_array(f, "battr", self.battr)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            request: r#"{"native":{"ver":"1.2"}}"#,
            ver: "1.2",
            api: &[ApiFramework::MRAID_2],
            battr: &[CreativeAttribute::AUDIO_AUTOPLAY],
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}
