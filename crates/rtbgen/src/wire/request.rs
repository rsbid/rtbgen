use core::fmt;

use super::{App, Device, ImpList, Regs, Site, Source, User};
use crate::code::AuctionType;
use crate::currency::Currency;
use crate::ids::RequestId;
use crate::raw::RawJson;
use crate::seed::Seed;
use crate::version::SpecVersion;
use crate::write::{
    Object, field_id, object, object_field, option_currency_array, option_number, option_raw,
    option_string_array,
};

/// The top-level bid request.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RequestModel {
    id: RequestId,
    imp: ImpList,
    site: Option<Site>,
    app: Option<App>,
    device: Option<Device>,
    user: Option<User>,
    at: Option<AuctionType>,
    tmax: Option<u32>,
    cur: &'static [Currency],
    bcat: &'static [&'static str],
    badv: &'static [&'static str],
    bapp: &'static [&'static str],
    source: Option<Source>,
    regs: Option<Regs>,
    ext: Option<RawJson>,
}

setters!(RequestModel {
    set_id => id: RequestId,
    set_imp => imp: ImpList,
    set_site => site: Option<Site>,
    set_app => app: Option<App>,
    set_device => device: Option<Device>,
    set_user => user: Option<User>,
    set_at => at: Option<AuctionType>,
    set_tmax => tmax: Option<u32>,
    set_cur => cur: &'static [Currency],
    set_bcat => bcat: &'static [&'static str],
    set_badv => badv: &'static [&'static str],
    set_bapp => bapp: &'static [&'static str],
    set_regs => regs: Option<Regs>,
    set_ext => ext: Option<RawJson>,
});

impl RequestModel {
    pub(crate) const fn impressions(&self) -> &ImpList {
        &self.imp
    }

    pub(crate) const fn impressions_mut(&mut self) -> &mut ImpList {
        &mut self.imp
    }

    pub(crate) fn write<W: fmt::Write>(&self, out: &mut W, spec: SpecVersion) -> fmt::Result {
        object(out, |f| self.write_fields(f, spec))
    }

    fn write_fields<W: fmt::Write>(&self, f: &mut Object<'_, W>, spec: SpecVersion) -> fmt::Result {
        field_id(f, "id", self.id)?;

        self.imp.write(f, spec)?;

        if let Some(site) = &self.site {
            object_field(f, "site", |f| site.write(f))?;
        }
        if let Some(app) = &self.app {
            object_field(f, "app", |f| app.write(f))?;
        }
        if let Some(device) = &self.device {
            object_field(f, "device", |f| device.write(f))?;
        }
        if let Some(user) = &self.user {
            object_field(f, "user", |f| user.write(f, spec))?;
        }

        option_number(f, "at", self.at)?;
        option_number(f, "tmax", self.tmax)?;
        option_currency_array(f, "cur", self.cur)?;
        option_string_array(f, "bcat", self.bcat)?;
        option_string_array(f, "badv", self.badv)?;
        option_string_array(f, "bapp", self.bapp)?;

        if let Some(source) = &self.source {
            object_field(f, "source", |f| source.write(f))?;
        }
        if let Some(regs) = &self.regs {
            object_field(f, "regs", |f| regs.write(f, spec))?;
        }

        option_raw(f, "ext", self.ext)
    }

    /// Builds the model with every field the writer can emit.
    ///
    /// The `Maximal` profile alternates the supply source, so this fixture
    /// carries both `site` and `app`; the profile drops one of them.
    pub(crate) fn maximal(seed: Seed) -> Self {
        let mut imps = ImpList::default();
        imps.push(&super::Imp::maximal());

        Self {
            id: RequestId::from_seed(seed),
            imp: imps,
            site: Some(Site::maximal()),
            app: Some(App::maximal()),
            device: Some(Device::maximal()),
            user: Some(User::maximal()),
            at: Some(AuctionType::SECOND_PRICE_PLUS),
            tmax: Some(1_000),
            cur: &[Currency::USD],
            bcat: &["IAB1"],
            badv: &["bad.example"],
            bapp: &["bad.app"],
            source: Some(Source::maximal()),
            regs: Some(Regs::maximal()),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}
