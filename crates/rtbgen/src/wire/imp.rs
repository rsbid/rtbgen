use core::fmt;

use super::MAX_IMPRESSIONS;
use super::context::Pmp;
use super::field::{WireCurrency, WireId, WireNumber};
use super::media::{Audio, Banner, Native, Video};
use crate::code::SsaiMode;
use crate::currency::Currency;
use crate::flag::Flag;
use crate::ids::ImpId;
use crate::money::Money;
use crate::raw::RawJson;
use crate::version::SpecVersion;
use crate::write::{Object, object, object_field, option_number, option_raw};

/// One impression opportunity.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Imp {
    id: WireId<ImpId>,
    banner: Option<Banner>,
    video: Option<Video>,
    audio: Option<Audio>,
    native: Option<Native>,
    pmp: Option<Pmp>,
    bidfloor: Option<WireNumber>,
    bidfloorcur: Option<WireCurrency>,
    secure: Option<Flag>,
    rwdd: Option<Flag>,
    ssai: Option<SsaiMode>,
    ext: Option<RawJson>,
}

setters!(Imp {
    set_id => id: WireId<ImpId>,
    set_banner => banner: Option<Banner>,
    set_video => video: Option<Video>,
    set_native => native: Option<Native>,
    set_pmp => pmp: Option<Pmp>,
    set_bidfloor => bidfloor: Option<WireNumber>,
    set_bidfloorcur => bidfloorcur: Option<WireCurrency>,
    set_secure => secure: Option<Flag>,
});

impl Imp {
    pub(crate) fn write<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        spec: SpecVersion,
    ) -> fmt::Result {
        self.id.write_field(f, "id")?;

        if let Some(banner) = &self.banner {
            object_field(f, "banner", |f| banner.write(f))?;
        }
        if let Some(video) = &self.video {
            object_field(f, "video", |f| video.write(f, spec))?;
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

        if let Some(bidfloor) = self.bidfloor {
            bidfloor.write_field(f, "bidfloor")?;
        }
        if let Some(bidfloorcur) = self.bidfloorcur {
            bidfloorcur.write_field(f, "bidfloorcur")?;
        }
        option_number(f, "secure", self.secure)?;

        if spec.is_v26() {
            option_number(f, "rwdd", self.rwdd)?;
            option_number(f, "ssai", self.ssai)?;
        }

        option_raw(f, "ext", self.ext)
    }

    /// Returns the impression identifier.
    pub(crate) const fn id(&self) -> WireId<ImpId> {
        self.id
    }

    /// Returns the banner of this impression, when it has one.
    pub(crate) const fn banner(&self) -> Option<&Banner> {
        self.banner.as_ref()
    }

    /// Returns the banner of this impression for mutation, when it has one.
    pub(crate) const fn banner_mut(&mut self) -> Option<&mut Banner> {
        self.banner.as_mut()
    }

    /// Returns the video of this impression for mutation, when it has one.
    pub(crate) const fn video_mut(&mut self) -> Option<&mut Video> {
        self.video.as_mut()
    }

    #[cfg(test)]
    pub(crate) const fn pmp_for_tests(self) -> Option<Pmp> {
        self.pmp
    }

    /// Builds the impression with every field the writer can emit.
    pub(crate) fn maximal() -> Self {
        Self {
            id: WireId::id(ImpId::new(1)),
            banner: Some(Banner::maximal()),
            video: Some(Video::maximal()),
            audio: Some(Audio::maximal()),
            native: Some(Native::maximal()),
            pmp: Some(Pmp::maximal()),
            bidfloor: Some(WireNumber::number(Money::from_scaled(15_000))),
            bidfloorcur: Some(WireCurrency::code(Currency::USD)),
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
    /// Extra copies of the last impression, written by `degrade`.
    ///
    /// The exchange cap is three impressions, so a larger body cannot be stored
    /// as a list of structs without sized storage. Repeating the last item keeps
    /// the model allocation-free while the writer still emits every object.
    extra: u16,
}

impl Default for ImpList {
    #[expect(clippy::large_stack_arrays, reason = "the wire model is allocation-free by design")]
    fn default() -> Self {
        Self { items: [Imp::default(); MAX_IMPRESSIONS], len: 0, extra: 0 }
    }
}

impl ImpList {
    /// Returns the number of stored impressions, without the repeated copies.
    pub(crate) const fn count(&self) -> u8 {
        self.len
    }

    /// Sets how many extra copies of the last impression the writer emits.
    pub(crate) const fn set_extra(&mut self, extra: u16) {
        self.extra = extra;
    }

    pub(crate) fn push(&mut self, imp: &Imp) {
        if let Some(slot) = self.items.get_mut(usize::from(self.len)) {
            *slot = *imp;
            self.len = self.len.saturating_add(1);
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Imp> {
        self.items.iter().take(usize::from(self.len))
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Imp> {
        let len = usize::from(self.len);
        self.items.iter_mut().take(len)
    }

    /// Returns the first impression, when the list is not empty.
    pub(crate) const fn first_mut(&mut self) -> Option<&mut Imp> {
        if self.len == 0 {
            return None;
        }
        self.items.first_mut()
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

        if self.len > 0
            && let Some(last) = self.items.get(usize::from(self.len).saturating_sub(1))
        {
            for _ in 0..self.extra {
                f.out().write_char(',')?;
                object(f.out(), |f| last.write(f, spec))?;
            }
        }

        f.out().write_char(']')
    }
}
