use core::fmt;

use crate::code::{DeviceType, LocationType};
use crate::flag::Flag;
use crate::ids::{AppId, BuyerUserId, PublisherId, SiteId, UserId};
use crate::raw::RawJson;
use crate::version::SpecVersion;
use crate::write::{
    Object, field_string, object_field, option_id, option_number, option_raw, option_string,
    option_string_array,
};

/// A website.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Site {
    id: Option<SiteId>,
    name: Option<&'static str>,
    domain: Option<&'static str>,
    cat: &'static [&'static str],
    page: Option<&'static str>,
    ref_page: Option<&'static str>,
    mobile: Option<Flag>,
    privacypolicy: Option<Flag>,
    publisher: Option<Publisher>,
    keywords: Option<&'static str>,
    ext: Option<RawJson>,
}

setters!(Site {
    set_id => id: Option<SiteId>,
    set_name => name: Option<&'static str>,
    set_domain => domain: Option<&'static str>,
    set_cat => cat: &'static [&'static str],
    set_page => page: Option<&'static str>,
    set_ref_page => ref_page: Option<&'static str>,
    set_mobile => mobile: Option<Flag>,
    set_privacypolicy => privacypolicy: Option<Flag>,
    set_publisher => publisher: Option<Publisher>,
    set_keywords => keywords: Option<&'static str>,
    set_ext => ext: Option<RawJson>,
});

impl Site {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_id(f, "id", self.id)?;
        option_string(f, "name", self.name)?;
        option_string(f, "domain", self.domain)?;
        option_string_array(f, "cat", self.cat)?;
        option_string(f, "page", self.page)?;
        option_string(f, "ref", self.ref_page)?;
        option_number(f, "mobile", self.mobile)?;
        option_number(f, "privacypolicy", self.privacypolicy)?;

        if let Some(publisher) = &self.publisher {
            object_field(f, "publisher", |f| publisher.write(f))?;
        }

        option_string(f, "keywords", self.keywords)?;
        option_raw(f, "ext", self.ext)
    }

    /// Builds the smallest site that can be a second supply source.
    pub(crate) const fn minimal() -> Self {
        Self {
            id: Some(SiteId::new(0)),
            name: None,
            domain: Some("site.example"),
            cat: &[],
            page: None,
            ref_page: None,
            mobile: None,
            privacypolicy: None,
            publisher: None,
            keywords: None,
            ext: None,
        }
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            id: Some(SiteId::new(1)),
            name: Some("Example Site"),
            domain: Some("site.example"),
            cat: &["IAB1"],
            page: Some("https://site.example/page"),
            ref_page: Some("https://search.example/"),
            mobile: Some(Flag::Yes),
            privacypolicy: Some(Flag::Yes),
            publisher: Some(Publisher::maximal()),
            keywords: Some("news,sports"),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// An application.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct App {
    id: Option<AppId>,
    name: Option<&'static str>,
    bundle: Option<&'static str>,
    storeurl: Option<&'static str>,
    domain: Option<&'static str>,
    cat: &'static [&'static str],
    ver: Option<&'static str>,
    publisher: Option<Publisher>,
    keywords: Option<&'static str>,
    ext: Option<RawJson>,
}

setters!(App {
    set_id => id: Option<AppId>,
    set_name => name: Option<&'static str>,
    set_bundle => bundle: Option<&'static str>,
    set_storeurl => storeurl: Option<&'static str>,
    set_domain => domain: Option<&'static str>,
    set_cat => cat: &'static [&'static str],
    set_ver => ver: Option<&'static str>,
    set_publisher => publisher: Option<Publisher>,
    set_keywords => keywords: Option<&'static str>,
    set_ext => ext: Option<RawJson>,
});

impl App {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_id(f, "id", self.id)?;
        option_string(f, "name", self.name)?;
        option_string(f, "bundle", self.bundle)?;
        option_string(f, "storeurl", self.storeurl)?;
        option_string(f, "domain", self.domain)?;
        option_string_array(f, "cat", self.cat)?;
        option_string(f, "ver", self.ver)?;

        if let Some(publisher) = &self.publisher {
            object_field(f, "publisher", |f| publisher.write(f))?;
        }

        option_string(f, "keywords", self.keywords)?;
        option_raw(f, "ext", self.ext)
    }

    /// Builds the smallest app that can be a second supply source.
    pub(crate) const fn minimal() -> Self {
        Self {
            id: Some(AppId::new(0)),
            name: None,
            bundle: Some("com.example.app"),
            storeurl: None,
            domain: None,
            cat: &[],
            ver: None,
            publisher: None,
            keywords: None,
            ext: None,
        }
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            id: Some(AppId::new(3)),
            name: Some("Example App"),
            bundle: Some("com.example.app"),
            storeurl: Some("https://apps.example/app"),
            domain: Some("app.example"),
            cat: &["IAB1"],
            ver: Some("1.2.3"),
            publisher: Some(Publisher::maximal()),
            keywords: Some("games"),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// The publisher of a site or an app.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Publisher {
    id: Option<PublisherId>,
    name: Option<&'static str>,
    domain: Option<&'static str>,
    cat: &'static [&'static str],
    ext: Option<RawJson>,
}

setters!(Publisher {
    set_id => id: Option<PublisherId>,
    set_name => name: Option<&'static str>,
    set_domain => domain: Option<&'static str>,
    set_cat => cat: &'static [&'static str],
});

impl Publisher {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_id(f, "id", self.id)?;
        option_string(f, "name", self.name)?;
        option_string(f, "domain", self.domain)?;
        option_string_array(f, "cat", self.cat)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            id: Some(PublisherId::new(2)),
            name: Some("Example Publisher"),
            domain: Some("publisher.example"),
            cat: &["IAB1"],
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// The user's device.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Device {
    ua: Option<&'static str>,
    ip: Option<&'static str>,
    geo: Option<Geo>,
    ifa: Option<&'static str>,
    dnt: Option<Flag>,
    lmt: Option<Flag>,
    devicetype: Option<DeviceType>,
    make: Option<&'static str>,
    model: Option<&'static str>,
    os: Option<&'static str>,
    osv: Option<&'static str>,
    language: Option<&'static str>,
    ext: Option<RawJson>,
}

setters!(Device {
    set_ua => ua: Option<&'static str>,
    set_ip => ip: Option<&'static str>,
    set_geo => geo: Option<Geo>,
    set_ifa => ifa: Option<&'static str>,
    set_dnt => dnt: Option<Flag>,
    set_lmt => lmt: Option<Flag>,
    set_devicetype => devicetype: Option<DeviceType>,
    set_make => make: Option<&'static str>,
    set_model => model: Option<&'static str>,
    set_os => os: Option<&'static str>,
    set_osv => osv: Option<&'static str>,
    set_language => language: Option<&'static str>,
});

impl Device {
    /// Returns the geo location, when the profile emitted one.
    pub(crate) const fn geo(&self) -> Option<&Geo> {
        self.geo.as_ref()
    }

    /// Returns the geo location for mutation, when the profile emitted one.
    pub(crate) const fn geo_mut(&mut self) -> Option<&mut Geo> {
        self.geo.as_mut()
    }

    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_string(f, "ua", self.ua)?;
        option_string(f, "ip", self.ip)?;

        if let Some(geo) = &self.geo {
            object_field(f, "geo", |f| geo.write(f))?;
        }

        option_string(f, "ifa", self.ifa)?;
        option_number(f, "dnt", self.dnt)?;
        option_number(f, "lmt", self.lmt)?;
        option_number(f, "devicetype", self.devicetype)?;
        option_string(f, "make", self.make)?;
        option_string(f, "model", self.model)?;
        option_string(f, "os", self.os)?;
        option_string(f, "osv", self.osv)?;
        option_string(f, "language", self.language)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            ua: Some("Test Agent/1.0"),
            ip: Some("203.0.113.7"),
            geo: Some(Geo::maximal()),
            ifa: Some("A1B2C3D4-0000-4000-8000-000000000001"),
            dnt: Some(Flag::No),
            lmt: Some(Flag::No),
            devicetype: Some(DeviceType::PERSONAL_COMPUTER),
            make: Some("Apple"),
            model: Some("iPhone15,2"),
            os: Some("iOS"),
            osv: Some("18.1"),
            language: Some("en"),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// The user's geographic location.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Geo {
    country: Option<&'static str>,
    region: Option<&'static str>,
    city: Option<&'static str>,
    zip: Option<&'static str>,
    type_code: Option<LocationType>,
    utcoffset: Option<i16>,
    ext: Option<RawJson>,
}

setters!(Geo {
    set_country => country: Option<&'static str>,
    set_region => region: Option<&'static str>,
    set_city => city: Option<&'static str>,
    set_zip => zip: Option<&'static str>,
    set_type_code => type_code: Option<LocationType>,
    set_utcoffset => utcoffset: Option<i16>,
});

impl Geo {
    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_string(f, "country", self.country)?;
        option_string(f, "region", self.region)?;
        option_string(f, "city", self.city)?;
        option_string(f, "zip", self.zip)?;
        option_number(f, "type", self.type_code)?;
        option_number(f, "utcoffset", self.utcoffset)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            country: Some("DEU"),
            region: Some("DE-BE"),
            city: Some("DEBER"),
            zip: Some("10115"),
            type_code: Some(LocationType::IP_ADDRESS),
            utcoffset: Some(60),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// The user.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct User {
    id: Option<UserId>,
    buyeruid: Option<BuyerUserId>,
    consent: Option<&'static str>,
    legacy_consent: Option<&'static str>,
    ext: Option<RawJson>,
}

setters!(User {
    set_id => id: Option<UserId>,
    set_buyeruid => buyeruid: Option<BuyerUserId>,
    set_consent => consent: Option<&'static str>,
    set_legacy_consent => legacy_consent: Option<&'static str>,
    set_ext => ext: Option<RawJson>,
});

impl User {
    /// Returns the TCF consent string, when the profile emitted one.
    pub(crate) const fn consent(&self) -> Option<&'static str> {
        self.consent
    }

    /// Returns the 2.5 `ext.consent` string, when the profile emitted one.
    pub(crate) const fn legacy_consent(&self) -> Option<&'static str> {
        self.legacy_consent
    }

    /// Returns whether the user would write an empty object.
    pub(crate) const fn is_empty(&self) -> bool {
        self.id.is_none()
            && self.buyeruid.is_none()
            && self.consent.is_none()
            && self.legacy_consent.is_none()
            && self.ext.is_none()
    }

    pub(crate) fn write<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        spec: SpecVersion,
    ) -> fmt::Result {
        option_id(f, "id", self.id)?;
        option_id(f, "buyeruid", self.buyeruid)?;

        if spec.is_v26() {
            option_string(f, "consent", self.consent)?;

            return option_raw(f, "ext", self.ext);
        }

        // OpenRTB 2.5 carried `consent` in `user.ext`; 2.6 promoted it.
        if let Some(consent) = self.legacy_consent {
            return object_field(f, "ext", |f| field_string(f, "consent", consent));
        }

        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            id: Some(UserId::new(4)),
            buyeruid: Some(BuyerUserId::new(5)),
            consent: Some(crate::privacy::TCF_CONSENT_GRANTED),
            legacy_consent: Some(crate::privacy::TCF_CONSENT_GRANTED),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}
