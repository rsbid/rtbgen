use core::fmt;

use super::MAX_IMPRESSIONS;
use super::context::Pmp;
use super::media::{Audio, Banner, Native, Video};
use crate::code::SsaiMode;
use crate::currency::Currency;
use crate::flag::Flag;
use crate::ids::ImpId;
use crate::money::Money;
use crate::raw::RawJson;
use crate::version::SpecVersion;
use crate::write::{
    Object, field_id, object, object_field, option_currency, option_money, option_number,
    option_raw,
};

/// One impression opportunity.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Imp {
    id: ImpId,
    banner: Option<Banner>,
    video: Option<Video>,
    audio: Option<Audio>,
    native: Option<Native>,
    pmp: Option<Pmp>,
    bidfloor: Option<Money>,
    bidfloorcur: Option<Currency>,
    secure: Option<Flag>,
    rwdd: Option<Flag>,
    ssai: Option<SsaiMode>,
    ext: Option<RawJson>,
}

setters!(Imp {
    set_id => id: ImpId,
    set_banner => banner: Option<Banner>,
    set_bidfloor => bidfloor: Option<Money>,
    set_bidfloorcur => bidfloorcur: Option<Currency>,
    set_secure => secure: Option<Flag>,
});

impl Imp {
    pub(crate) fn write<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        spec: SpecVersion,
    ) -> fmt::Result {
        field_id(f, "id", self.id)?;

        if let Some(banner) = &self.banner {
            object_field(f, "banner", |f| banner.write(f))?;
        }
        if let Some(video) = &self.video {
            object_field(f, "video", |f| video.write(f))?;
        }
        if let Some(audio) = &self.audio {
            object_field(f, "audio", |f| audio.write(f))?;
        }
        if let Some(native) = &self.native {
            object_field(f, "native", |f| native.write(f))?;
        }
        if let Some(pmp) = &self.pmp {
            object_field(f, "pmp", |f| pmp.write(f))?;
        }

        option_money(f, "bidfloor", self.bidfloor)?;
        option_currency(f, "bidfloorcur", self.bidfloorcur)?;
        option_number(f, "secure", self.secure)?;

        if spec.is_v26() {
            option_number(f, "rwdd", self.rwdd)?;
            option_number(f, "ssai", self.ssai)?;
        }

        option_raw(f, "ext", self.ext)
    }

    #[cfg(test)]
    pub(super) fn maximal() -> Self {
        Self {
            id: ImpId::new(1),
            banner: Some(Banner::maximal()),
            video: Some(Video::maximal()),
            audio: Some(Audio::maximal()),
            native: Some(Native::maximal()),
            pmp: Some(Pmp::maximal()),
            bidfloor: Some(Money::from_scaled(15_000)),
            bidfloorcur: Some(Currency::USD),
            secure: Some(Flag::Yes),
            rwdd: Some(Flag::Yes),
            ssai: Some(SsaiMode::SERVER_SIDE),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// A fixed-capacity list of impressions.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ImpList {
    items: [Imp; MAX_IMPRESSIONS],
    len: u8,
}

impl Default for ImpList {
    #[expect(
        clippy::large_stack_arrays,
        reason = "the wire model is allocation-free by design"
    )]
    fn default() -> Self {
        Self {
            items: [Imp::default(); MAX_IMPRESSIONS],
            len: 0,
        }
    }
}

impl ImpList {
    pub(crate) fn push(&mut self, imp: &Imp) {
        if let Some(slot) = self.items.get_mut(usize::from(self.len)) {
            *slot = *imp;
            self.len = self.len.saturating_add(1);
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Imp> {
        self.items.iter().take(usize::from(self.len))
    }

    pub(crate) fn write<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        spec: SpecVersion,
    ) -> fmt::Result {
        f.key("imp")?;
        f.out().write_char('[')?;

        let mut first = true;
        for imp in self.iter() {
            if first {
                first = false;
            } else {
                f.out().write_char(',')?;
            }
            object(f.out(), |f| imp.write(f, spec))?;
        }

        f.out().write_char(']')
    }
}
